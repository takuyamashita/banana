//! キューから受け取ったメッセージの処理。Lambda 本体(main.rs)とローカル用ポーラー(bin/local_poller.rs)で共有する

use anyhow::Context as _;
use payroll_domain::payslip::PayslipId;
use payroll_domain::staff::StaffId;
use payroll_infrastructure::messaging::payloads::PayslipFinalizedPayload;
use payroll_usecase::payslip::{RequestPayoutInput, RequestPayoutResult, RequestPayoutUseCase};
use platform_kernel::Money;
use platform_messaging::envelope::OutboxEnvelope;

pub use platform_messaging::consumer::{Message, handle_batch};

/// Lambda の初期化(run の外)で1回だけ作る
pub struct Deps {
    pub usecase: RequestPayoutUseCase,
}

impl Deps {
    pub async fn build() -> anyhow::Result<Self> {
        let config = payroll_bootstrap::load_config()?;
        let aws = payroll_bootstrap::aws_config().await;
        let pool = payroll_bootstrap::connect_db(&config, &aws).await?;
        Ok(Self { usecase: payroll_bootstrap::build_request_payout(&config, &aws, &pool).await? })
    }
}

/// このキューの受け手が扱う出来事
pub enum Received {
    /// 給与明細が確定した。振込を依頼する
    PayslipFinalized { event_id: i64, input: RequestPayoutInput },
    /// この受け手には関係のない出来事(読み飛ばす)
    Other { event_id: i64, event_type: String },
}

/// メッセージの本文を読み、出来事の種類ごとに振り分ける。
/// 知らない種類は読み飛ばす(同じキューに他の受け手向けの出来事が増えても、誤って振り込まない)
pub fn decode(body: &str) -> anyhow::Result<Received> {
    let envelope: OutboxEnvelope<serde_json::Value> =
        serde_json::from_str(body).context("invalid message body")?;
    let event_id = envelope.event_id;
    if envelope.event_type != PayslipFinalizedPayload::EVENT_TYPE {
        return Ok(Received::Other { event_id, event_type: envelope.event_type });
    }
    let ev: PayslipFinalizedPayload =
        serde_json::from_value(envelope.payload).context("invalid payslip.finalized payload")?;
    let payslip_id = PayslipId::from_i64(ev.payslip_id)?;
    Ok(Received::PayslipFinalized {
        event_id,
        input: RequestPayoutInput {
            payslip_id,
            staff_id: StaffId::from_i64(ev.staff_id)?,
            total: Money::from_yen(ev.total_yen)?,
            idempotency_key: idempotency_key(payslip_id),
        },
    })
}

/// 振込先に渡す冪等キー。1つの給与明細の振込は1回なので、給与明細番号から決める。
/// outbox の連番を使うと、DB を過去の時点に戻したときに同じ番号が別の確定に振られ、
/// 振込先が「依頼済み」と答えて振り込まれないことがある
fn idempotency_key(payslip_id: PayslipId) -> String {
    format!("payroll-payslip-{}-finalized", payslip_id.as_i64())
}

/// メッセージ1件を処理する。
///
/// SQS は同じメッセージを2回以上届けることがある。振込依頼は給与明細ごとに記録するので、
/// 依頼済みなら何もしない。依頼と記録の間で落ちた場合に備え、振込先には冪等キーを渡す。
/// 振込先に断られたときは記録して成功扱いにする(やり直しても通らない)。
/// 振込先につながらないときは失敗を返し、キューからの再配信でやり直す
pub async fn process(deps: &Deps, body: &str) -> anyhow::Result<()> {
    let (event_id, input) = match decode(body)? {
        Received::PayslipFinalized { event_id, input } => (event_id, input),
        Received::Other { event_id, event_type } => {
            tracing::info!(event_id, event_type, "not for this consumer, skipped");
            return Ok(());
        }
    };
    let payslip_id = input.payslip_id.as_i64();
    match deps.usecase.execute(input).await? {
        RequestPayoutResult::Accepted => {
            tracing::info!(event_id, payslip_id, "payout requested");
        }
        RequestPayoutResult::Rejected { reason } => {
            tracing::error!(event_id, payslip_id, reason, "payout rejected");
        }
        RequestPayoutResult::AlreadyRequested => {
            tracing::info!(event_id, payslip_id, "payout already requested, skipped");
        }
    }
    Ok(())
}

/// ローカルのポーラーが使う受け手
pub struct Consumer(pub Deps);

#[async_trait::async_trait]
impl platform_messaging::consumer::Handler for Consumer {
    async fn handle(&self, body: &str) -> anyhow::Result<()> {
        process(&self.0, body).await
    }
}

#[cfg(test)]
mod tests {
    use aws_lambda_events::sqs::SqsEvent;

    use super::*;

    #[test]
    fn sample_event_is_decoded_as_a_payout_request() {
        // cargo lambda invoke に渡すサンプル(events/)が、今のキューの形で読めること
        let event: SqsEvent =
            serde_json::from_str(include_str!("../events/sqs-payslip-finalized.json")).unwrap();
        let body = event.records[0].body.as_deref().unwrap();

        let Received::PayslipFinalized { event_id, input } = decode(body).unwrap() else {
            panic!("給与確定として読めるはず");
        };
        assert_eq!(event_id, 900_001);
        assert_eq!(input.payslip_id.as_i64(), 1);
        assert_eq!(input.total.as_yen(), 1_000);
        // 冪等キーは outbox の連番ではなく給与明細番号から決まる
        // gitleaks の generic-api-key が「key と文字列の比較」を API キーと誤検知するので、この行だけ外す
        assert_eq!(input.idempotency_key, "payroll-payslip-1-finalized"); // gitleaks:allow
    }

    #[test]
    fn events_of_other_types_are_skipped() {
        let body = r#"{"event_id":1,"event_type":"payslip.corrected","aggregate_type":"payslip",
                       "aggregate_id":1,"payload":{"staff_id":1,"total_yen":1000}}"#;
        assert!(
            matches!(decode(body).unwrap(), Received::Other { event_type, .. } if event_type == "payslip.corrected")
        );
    }

    #[test]
    fn a_body_without_event_type_is_an_error() {
        // 種類の分からないメッセージは、黙って振込として扱わない
        let body = r#"{"event_id":1,"payload":{"payslip_id":1,"staff_id":1,"pay_year":2026,"pay_month":9,"total_yen":1000}}"#;
        assert!(decode(body).is_err());
    }

    #[test]
    fn the_payload_written_by_the_outbox_is_read_by_this_consumer() {
        // outbox が書く形(PayslipFinalizedPayload)と、受け手が読む形が合っていること
        let payload = PayslipFinalizedPayload {
            payslip_id: 3,
            staff_id: 2,
            pay_year: 2026,
            pay_month: 9,
            total_yen: 12_000,
            finalized_at: "2026-09-30T10:00:00Z".into(),
        };
        let body = serde_json::to_string(&OutboxEnvelope {
            event_id: 10,
            event_type: PayslipFinalizedPayload::EVENT_TYPE.to_owned(),
            aggregate_type: "payslip".to_owned(),
            aggregate_id: 3,
            payload,
        })
        .unwrap();
        let Received::PayslipFinalized { input, .. } = decode(&body).unwrap() else { panic!() };
        assert_eq!((input.payslip_id.as_i64(), input.staff_id.as_i64()), (3, 2));
    }
}
