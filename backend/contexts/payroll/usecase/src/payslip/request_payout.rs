use std::sync::Arc;

use payroll_domain::staff::StaffId;
use platform_kernel::Money;

use crate::UseCaseError;
use crate::ports::payout_gateway::PayoutGateway;

/// 確定した給与を、派遣社員の口座に振り込むよう依頼する
pub struct RequestPayoutUseCase {
    payout_gateway: Arc<dyn PayoutGateway>,
}

impl RequestPayoutUseCase {
    #[must_use]
    pub fn new(payout_gateway: Arc<dyn PayoutGateway>) -> Self {
        Self { payout_gateway }
    }

    /// 派遣社員に支給額を振り込むよう依頼する。
    /// `idempotency_key` は給与確定ごとの番号で、同じ給与確定について何度依頼しても振込は1回になる
    pub async fn execute(
        &self,
        staff_id: StaffId,
        total: Money,
        idempotency_key: &str,
    ) -> Result<(), UseCaseError> {
        self.payout_gateway.request_transfer(staff_id, total, idempotency_key).await?;
        Ok(())
    }
}
