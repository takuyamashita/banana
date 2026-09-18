use async_trait::async_trait;
use payroll_domain::staff::StaffId;
use payroll_usecase::ports::payout_gateway::{PayoutError, PayoutGateway, PayoutId};
use platform_kernel::Money;

/// ローカル用。振込APIの代わりにログへ出すだけ。config の payout.provider = "logging" で選ぶ
pub struct LoggingPayoutGateway;

#[async_trait]
impl PayoutGateway for LoggingPayoutGateway {
    async fn request_transfer(
        &self,
        staff_id: StaffId,
        amount: Money,
        idempotency_key: &str,
    ) -> Result<PayoutId, PayoutError> {
        tracing::info!(
            staff_id = staff_id.as_i64(),
            amount_yen = amount.as_yen(),
            idempotency_key,
            "payout requested (logging gateway)"
        );
        Ok(PayoutId(format!("local-{idempotency_key}")))
    }
}
