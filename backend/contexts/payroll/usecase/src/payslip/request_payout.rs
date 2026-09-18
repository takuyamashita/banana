use std::sync::Arc;

use payroll_domain::staff::StaffId;
use platform_kernel::Money;

use crate::UseCaseError;
use crate::ports::payout_gateway::PayoutGateway;

// SQSのconsumerから呼ばれるユースケース
pub struct RequestPayoutUseCase {
    payout_gateway: Arc<dyn PayoutGateway>,
}

impl RequestPayoutUseCase {
    #[must_use]
    pub fn new(payout_gateway: Arc<dyn PayoutGateway>) -> Self {
        Self { payout_gateway }
    }

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
