use std::sync::Arc;

use payroll_domain::payslip::{NewPayslip, PayPeriod, PayslipId, PayslipLine};
use payroll_domain::staff::StaffId;

use crate::UseCaseError;
use crate::ports::repository::{PayslipRepository, StaffRepository};

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
pub struct FinalizePayslipUseCase {
    repository: Arc<dyn PayslipRepository>,
    staff_repository: Arc<dyn StaffRepository>,
}

impl FinalizePayslipUseCase {
    #[must_use]
    pub fn new(
        repository: Arc<dyn PayslipRepository>,
        staff_repository: Arc<dyn StaffRepository>,
    ) -> Self {
        Self { repository, staff_repository }
    }

    /// 給与を確定し、振られた給与明細番号を返す。
    ///
    /// 派遣社員が登録されていなければ `InvalidInput`、その月の給与が確定済みなら `Conflict`、
    /// 明細行がない・稼働時間が不正などの業務ルール違反なら `InvalidInput` になる
    pub async fn execute(&self, input: FinalizePayslipInput) -> Result<PayslipId, UseCaseError> {
        if self.staff_repository.find(input.staff_id).await?.is_none() {
            return Err(UseCaseError::InvalidInput("派遣社員が存在しません".into()));
        }

        let existing = self.repository.list_by_staff(input.staff_id).await?;
        if existing.iter().any(|p| p.period() == input.period) {
            return Err(UseCaseError::Conflict("この月の給与明細は既に確定しています".into()));
        }

        let mut new = NewPayslip::draft(input.staff_id, input.period, input.lines)?;

        new.finalize()?;

        let payslip = self.repository.insert(&mut new).await?;

        Ok(payslip.id())
    }
}
