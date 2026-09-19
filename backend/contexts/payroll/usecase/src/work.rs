//! 勤怠で承認された稼働。給与明細の明細行の稼働の元にする

use std::sync::Arc;

use payroll_domain::payslip::PayPeriod;
use payroll_domain::staff::StaffId;
use payroll_domain::work::ApprovedWork;

use crate::UseCaseError;
use crate::ports::database::Database;
use crate::ports::repository::ApprovedWorkRepository;

/// 勤怠で勤務表が承認されたことを記録する
pub struct RecordApprovedWorkUseCase {
    work: Arc<dyn ApprovedWorkRepository>,
    db: Arc<dyn Database>,
}

impl RecordApprovedWorkUseCase {
    #[must_use]
    pub fn new(work: Arc<dyn ApprovedWorkRepository>, db: Arc<dyn Database>) -> Self {
        Self { work, db }
    }

    pub async fn execute(&self, work: ApprovedWork) -> Result<(), UseCaseError> {
        let mut db = self.db.connection().await?;
        Ok(self.work.save(&mut db, &work).await?)
    }
}

/// 管理者が、派遣社員のその月の承認された稼働を見る(給与明細の明細行に入れる)
pub struct GetApprovedWorkUseCase {
    work: Arc<dyn ApprovedWorkRepository>,
}

impl GetApprovedWorkUseCase {
    #[must_use]
    pub fn new(work: Arc<dyn ApprovedWorkRepository>) -> Self {
        Self { work }
    }

    pub async fn execute(
        &self,
        staff_id: StaffId,
        period: PayPeriod,
    ) -> Result<Option<ApprovedWork>, UseCaseError> {
        Ok(self.work.find(staff_id, period).await?)
    }
}
