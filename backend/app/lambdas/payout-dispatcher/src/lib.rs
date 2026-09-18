//! イベント1件の処理。Lambda 本体(main.rs)とローカル用ポーラー(bin/local_poller.rs)で共有する

use anyhow::Context as _;
use payroll_domain::staff::StaffId;
use payroll_infrastructure::messaging::envelope::OutboxEnvelope;
use payroll_infrastructure::messaging::payloads::PayslipFinalizedPayload;
use payroll_infrastructure::messaging::processed_events::ProcessedEvents;
use payroll_usecase::payslip::RequestPayoutUseCase;
use platform_kernel::Money;

/// Lambda の初期化(run の外)で1回だけ作る
pub struct Deps {
    pub usecase: RequestPayoutUseCase,
    pub processed: ProcessedEvents,
}

impl Deps {
    pub async fn build() -> anyhow::Result<Self> {
        let config = bootstrap::load_config()?;
        let aws = bootstrap::aws_config().await;
        let pool = bootstrap::connect_db(&config, &aws).await?;
        Ok(Self {
            usecase: bootstrap::build_request_payout(&config),
            processed: ProcessedEvents::new(pool),
        })
    }
}

/// SQS は at-least-once なので、処理済みイベントIDを記録して二重処理を弾く。
/// 振込と記録の間で落ちた場合に備え、振込APIには event_id を冪等キーとして渡す
pub async fn process(deps: &Deps, body: &str) -> anyhow::Result<()> {
    // relay が付けた封筒。event_id は outbox.id
    let envelope: OutboxEnvelope<PayslipFinalizedPayload> =
        serde_json::from_str(body).context("invalid message body")?;
    let ev = envelope.payload;

    if deps.processed.already_done(envelope.event_id).await? {
        tracing::info!(event_id = envelope.event_id, "already processed, skipped");
        return Ok(());
    }

    let staff_id = StaffId::from_i64(ev.staff_id)?;
    let total = Money::from_yen(ev.total_yen)?;
    deps.usecase.execute(staff_id, total, &envelope.event_id.to_string()).await?;

    deps.processed.mark(envelope.event_id).await?;
    tracing::info!(event_id = envelope.event_id, staff_id = ev.staff_id, "payout requested");
    Ok(())
}
