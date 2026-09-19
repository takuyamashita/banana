use std::sync::Arc;

use payroll_domain::payslip::{Payslip, PayslipId};

use crate::UseCaseError;
use crate::ports::clock::Clock;
use crate::ports::database::Database;
use crate::ports::events::{EventOutbox, PayrollEvent};
use crate::ports::repository::PayslipRepository;

/// 管理者が、作成中の給与明細を確定する。
///
/// 確定した給与明細は変更できず、支給額が決まったことが振込に伝わる。
/// 確定できるのは作成中の給与明細だけで、1つの給与明細は1回しか確定できない
pub struct FinalizePayslipUseCase {
    payslips: Arc<dyn PayslipRepository>,
    outbox: Arc<dyn EventOutbox>,
    db: Arc<dyn Database>,
    clock: Arc<dyn Clock>,
}

impl FinalizePayslipUseCase {
    #[must_use]
    pub fn new(
        payslips: Arc<dyn PayslipRepository>,
        outbox: Arc<dyn EventOutbox>,
        db: Arc<dyn Database>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { payslips, outbox, db, clock }
    }

    /// 給与明細を今の日時で確定する。
    ///
    /// 給与明細がなければ `NotFound`、既に確定済みなら `FailedPrecondition` になる。
    /// 同じ給与明細を同時に確定しようとしても、確定されるのは1回だけ
    pub async fn execute(&self, id: PayslipId) -> Result<(), UseCaseError> {
        let mut tx = self.db.transaction().await?;
        let payslip =
            self.payslips.find_for_update(&mut tx, id).await?.ok_or(UseCaseError::NotFound)?;
        let Payslip::Draft(draft) = payslip else {
            return Err(UseCaseError::FailedPrecondition(
                "確定済みの給与明細は確定できません".into(),
            ));
        };

        let (finalized, event) = draft.finalize(self.clock.now());
        self.payslips.update(&mut tx, &finalized.into()).await?;
        self.outbox.append(&mut tx, PayrollEvent::Payslip(event)).await?;
        tx.commit().await?;
        Ok(())
    }
}
