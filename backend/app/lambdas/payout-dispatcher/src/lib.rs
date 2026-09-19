//! キューから受け取ったメッセージの処理。Lambda 本体(main.rs)とローカル用ポーラー(bin/local_poller.rs)で共有する

use std::collections::HashSet;

use anyhow::Context as _;
use payroll_domain::payslip::PayslipId;
use payroll_domain::staff::StaffId;
use payroll_infrastructure::messaging::envelope::OutboxEnvelope;
use payroll_infrastructure::messaging::payloads::PayslipFinalizedPayload;
use payroll_usecase::payslip::{RequestPayoutInput, RequestPayoutResult, RequestPayoutUseCase};
use platform_kernel::Money;

/// Lambda の初期化(run の外)で1回だけ作る
pub struct Deps {
    pub usecase: RequestPayoutUseCase,
}

impl Deps {
    pub async fn build() -> anyhow::Result<Self> {
        let config = bootstrap::load_config()?;
        let aws = bootstrap::aws_config().await;
        let pool = bootstrap::connect_db(&config, &aws).await?;
        Ok(Self { usecase: bootstrap::build_request_payout(&config, &pool)? })
    }
}

/// 給与確定のメッセージ1件を処理する。
///
/// SQS は同じメッセージを2回以上届けることがある。振込依頼は給与明細ごとに記録するので、
/// 依頼済みなら何もしない。依頼と記録の間で落ちた場合に備え、振込先には event_id を冪等キーとして渡す。
/// 振込先に断られたときは記録して成功扱いにする(やり直しても通らない)。
/// 振込先につながらないときは失敗を返し、キューからの再配信でやり直す
pub async fn process(deps: &Deps, body: &str) -> anyhow::Result<()> {
    // relay が付けた封筒。event_id は outbox.id
    let envelope: OutboxEnvelope<PayslipFinalizedPayload> =
        serde_json::from_str(body).context("invalid message body")?;
    let event_id = envelope.event_id;
    let ev = envelope.payload;

    let input = RequestPayoutInput {
        payslip_id: PayslipId::from_i64(ev.payslip_id)?,
        staff_id: StaffId::from_i64(ev.staff_id)?,
        total: Money::from_yen(ev.total_yen)?,
        idempotency_key: event_id.to_string(),
    };
    match deps.usecase.execute(input).await? {
        RequestPayoutResult::Accepted => {
            tracing::info!(event_id, payslip_id = ev.payslip_id, "payout requested");
        }
        RequestPayoutResult::Rejected { reason } => {
            tracing::error!(event_id, payslip_id = ev.payslip_id, reason, "payout rejected");
        }
        RequestPayoutResult::AlreadyRequested => {
            tracing::info!(
                event_id,
                payslip_id = ev.payslip_id,
                "payout already requested, skipped"
            );
        }
    }
    Ok(())
}

/// キューから受け取ったメッセージ1件
pub struct Message {
    /// メッセージ ID。処理に失敗した件をキューに返すときに使う
    pub id: String,
    /// FIFO のメッセージグループ(給与明細ごと)。同じグループの中では順に処理する
    pub group: String,
    pub body: String,
}

/// 受け取った順に処理し、失敗した件のメッセージ ID を返す。失敗した件だけがキューに戻る。
///
/// 同じグループの中では順序を守るため、ある件が失敗したら同じグループの後ろの件は処理せずに失敗として返す。
/// 別のグループ(別の給与明細)は、そのまま処理を続ける
pub async fn handle_batch(
    messages: &[Message],
    mut process: impl AsyncFnMut(&str) -> anyhow::Result<()>,
) -> Vec<String> {
    let mut failed_groups = HashSet::new();
    let mut failed = Vec::new();
    for message in messages {
        if failed_groups.contains(&message.group) {
            failed.push(message.id.clone());
            continue;
        }
        if let Err(err) = process(&message.body).await {
            tracing::warn!(message_id = %message.id, error = %err, "processing failed, will retry");
            failed_groups.insert(message.group.clone());
            failed.push(message.id.clone());
        }
    }
    failed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(id: &str, group: &str, body: &str) -> Message {
        Message { id: id.into(), group: group.into(), body: body.into() }
    }

    #[tokio::test]
    async fn a_failure_holds_back_only_the_rest_of_its_group() {
        let messages = [
            message("1", "payslip-1", "ok"),
            message("2", "payslip-2", "fail"),
            message("3", "payslip-1", "ok"),
            message("4", "payslip-2", "ok"),
            message("5", "payslip-3", "ok"),
        ];
        let mut processed = Vec::new();

        let failed = handle_batch(&messages, async |body: &str| {
            processed.push(body.to_owned());
            if body == "fail" { Err(anyhow::anyhow!("down")) } else { Ok(()) }
        })
        .await;

        // 失敗したグループの後ろの件(4)は処理せずに返し、他のグループは処理を続ける
        assert_eq!(failed, ["2", "4"]);
        assert_eq!(processed.len(), 4);
    }
}
