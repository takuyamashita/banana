use std::sync::Arc;

use payroll_domain::payout::{Payout, PayoutOutcome};
use payroll_domain::payslip::PayslipId;
use payroll_domain::staff::StaffId;
use platform_kernel::Money;

use crate::UseCaseError;
use crate::ports::database::Database;
use crate::ports::payout_gateway::{PayoutError, PayoutGateway};
use crate::ports::repository::{PayoutRepository, RepositoryError};

/// 確定した給与明細について、振込を依頼する内容
pub struct RequestPayoutInput {
    /// 支給額が決まった給与明細
    pub payslip_id: PayslipId,
    /// 振込を受ける派遣社員
    pub staff_id: StaffId,
    /// 振り込む金額(給与明細の支給額)
    pub total: Money,
    /// 給与確定ごとの番号。同じ給与確定について何度依頼しても、振込は1回になる
    pub idempotency_key: String,
}

/// 振込を依頼した結果
#[derive(Debug, PartialEq, Eq)]
pub enum RequestPayoutResult {
    /// 振込先が受け付けた
    Accepted,
    /// 振込先に断られた。理由とともに記録したので、管理者が確かめて対処する
    Rejected { reason: String },
    /// この給与明細の振込は既に依頼済みで、何もしなかった
    AlreadyRequested,
}

/// 確定した給与を、派遣社員の口座に振り込むよう依頼し、振込先の答えを記録する。
///
/// 1つの給与明細の振込は1回だけ依頼する。振込先に一時的につながらないときは、
/// 何も記録せずに失敗を返すので、時間をおいて依頼し直す
pub struct RequestPayoutUseCase {
    payouts: Arc<dyn PayoutRepository>,
    payout_gateway: Arc<dyn PayoutGateway>,
    db: Arc<dyn Database>,
}

impl RequestPayoutUseCase {
    #[must_use]
    pub fn new(
        payouts: Arc<dyn PayoutRepository>,
        payout_gateway: Arc<dyn PayoutGateway>,
        db: Arc<dyn Database>,
    ) -> Self {
        Self { payouts, payout_gateway, db }
    }

    /// 振込を依頼し、振込先が受け付けたか断ったかを記録する。
    ///
    /// 断られたときも記録して `Rejected` を返す(やり直しても通らないので失敗にはしない)。
    /// 振込先につながらないときは `Unavailable` になる
    pub async fn execute(
        &self,
        input: RequestPayoutInput,
    ) -> Result<RequestPayoutResult, UseCaseError> {
        if self.payouts.find_by_payslip(input.payslip_id).await?.is_some() {
            return Ok(RequestPayoutResult::AlreadyRequested);
        }

        let outcome = match self
            .payout_gateway
            .request_transfer(input.staff_id, input.total, &input.idempotency_key)
            .await
        {
            Ok(receipt) => PayoutOutcome::Accepted { receipt: receipt.0 },
            Err(PayoutError::Rejected(reason)) => PayoutOutcome::Rejected { reason },
            Err(err @ PayoutError::Unavailable(_)) => return Err(err.into()),
        };

        let payout = Payout::new(input.payslip_id, input.staff_id, input.total, outcome.clone());
        let mut db = self.db.connection().await?;
        match self.payouts.insert(&mut db, &payout).await {
            Ok(_) => {}
            // 同じ給与明細の依頼が先に記録されていた。振込先には同じ番号で依頼したので、振込は1回
            Err(RepositoryError::Conflict(_)) => return Ok(RequestPayoutResult::AlreadyRequested),
            Err(err) => return Err(err.into()),
        }

        Ok(match outcome {
            PayoutOutcome::Accepted { .. } => RequestPayoutResult::Accepted,
            PayoutOutcome::Rejected { reason } => RequestPayoutResult::Rejected { reason },
        })
    }
}
