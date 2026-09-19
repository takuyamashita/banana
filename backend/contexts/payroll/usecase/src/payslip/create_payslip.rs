use std::sync::Arc;

use payroll_domain::payslip::{PayPeriod, Payslip, PayslipId, PayslipLine};
use payroll_domain::staff::StaffId;

use crate::UseCaseError;
use crate::ports::database::Database;
use crate::ports::repository::{PayslipRepository, StaffRepository};

/// 給与明細の作成で管理者が入力する内容
pub struct CreatePayslipInput {
    /// 給与を受け取る派遣社員
    pub staff_id: StaffId,
    /// 対象月
    pub period: PayPeriod,
    /// 案件ごとの稼働と時給
    pub lines: Vec<PayslipLine>,
}

/// 管理者が、派遣社員1人の1か月分の給与明細を作成中として作る。
///
/// 作成中の給与明細は、管理者が内容を確かめてから確定する。
/// 同じ派遣社員・同じ月の給与明細は1つしか作れない
pub struct CreatePayslipUseCase {
    payslips: Arc<dyn PayslipRepository>,
    staff: Arc<dyn StaffRepository>,
    db: Arc<dyn Database>,
}

impl CreatePayslipUseCase {
    #[must_use]
    pub fn new(
        payslips: Arc<dyn PayslipRepository>,
        staff: Arc<dyn StaffRepository>,
        db: Arc<dyn Database>,
    ) -> Self {
        Self { payslips, staff, db }
    }

    /// 給与明細を作成中として作り、振られた給与明細番号を返す。
    ///
    /// 派遣社員が登録されていなければ `InvalidInput`、その月の給与明細が既にあれば `Conflict`、
    /// 明細行がない・稼働時間が不正などの業務ルール違反なら `InvalidInput` になる
    pub async fn execute(&self, input: CreatePayslipInput) -> Result<PayslipId, UseCaseError> {
        if self.staff.find(input.staff_id).await?.is_none() {
            return Err(UseCaseError::InvalidInput("派遣社員が存在しません".into()));
        }
        let existing = self.payslips.list_by_staff(input.staff_id).await?;
        if existing.iter().any(|p| p.content().period() == input.period) {
            return Err(UseCaseError::Conflict("この月の給与明細は既にあります".into()));
        }

        let draft = Payslip::draft(input.staff_id, input.period, input.lines)?;
        let mut db = self.db.connection().await?;
        Ok(self.payslips.insert(&mut db, &draft.into()).await?)
    }
}
