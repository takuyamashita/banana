use std::sync::Arc;

use payroll_domain::payslip::{NewPayslip, PayPeriod, PayslipId, PayslipLine};
use payroll_domain::staff::StaffId;

use crate::UseCaseError;
use crate::ports::events::{EventOutbox, PayrollEvent};
use crate::ports::repository::{PayslipRepository, StaffRepository};
use crate::ports::transaction::Transactions;

/// 給与確定で管理者が入力する内容
pub struct FinalizePayslipInput {
    /// 給与を確定する派遣社員
    pub staff_id: StaffId,
    /// 対象月
    pub period: PayPeriod,
    /// 案件ごとの稼働と時給
    pub lines: Vec<PayslipLine>,
}

/// 管理者が、派遣社員1人の1か月分の給与を確定する。
///
/// 確定した給与明細は変更できず、支給額が決まったことが振込に伝わる。
/// 同じ派遣社員・同じ月の給与は1回しか確定できない
pub struct FinalizePayslipUseCase<T: Transactions> {
    payslips: Arc<dyn PayslipRepository<T::Tx>>,
    staff: Arc<dyn StaffRepository<T::Tx>>,
    outbox: Arc<dyn EventOutbox<T::Tx>>,
    transactions: Arc<T>,
}

impl<T: Transactions> FinalizePayslipUseCase<T> {
    #[must_use]
    pub fn new(
        payslips: Arc<dyn PayslipRepository<T::Tx>>,
        staff: Arc<dyn StaffRepository<T::Tx>>,
        outbox: Arc<dyn EventOutbox<T::Tx>>,
        transactions: Arc<T>,
    ) -> Self {
        Self { payslips, staff, outbox, transactions }
    }

    /// 給与を確定し、振られた給与明細番号を返す。
    ///
    /// 派遣社員が登録されていなければ `InvalidInput`、その月の給与が確定済みなら `Conflict`、
    /// 明細行がない・稼働時間が不正などの業務ルール違反なら `InvalidInput` になる
    pub async fn execute(&self, input: FinalizePayslipInput) -> Result<PayslipId, UseCaseError> {
        if self.staff.find(input.staff_id).await?.is_none() {
            return Err(UseCaseError::InvalidInput("派遣社員が存在しません".into()));
        }
        let existing = self.payslips.list_by_staff(input.staff_id).await?;
        if existing.iter().any(|p| p.period() == input.period) {
            return Err(UseCaseError::Conflict("この月の給与明細は既に確定しています".into()));
        }

        let mut payslip = NewPayslip::draft(input.staff_id, input.period, input.lines)?;
        let finalized = payslip.finalize()?;

        let mut tx = self.transactions.begin().await?;
        let id = self.payslips.insert(&mut tx, &payslip).await?;
        self.outbox.append(&mut tx, PayrollEvent::Payslip { id, event: finalized }).await?;
        self.transactions.commit(tx).await?;

        Ok(id)
    }
}
