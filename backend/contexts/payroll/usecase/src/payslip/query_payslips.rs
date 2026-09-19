use std::sync::Arc;

use payroll_domain::payslip::{Payslip, PayslipId, PayslipStatus};
use payroll_domain::staff::StaffId;
use platform_kernel::{AuthenticatedUser, Role};

use crate::UseCaseError;
use crate::ports::repository::{PayslipRepository, StaffRepository};

/// 給与明細を見てよいのは、管理者と、その給与明細を受け取る派遣社員本人だけ
async fn can_view(
    staff_repository: &dyn StaffRepository,
    user: &AuthenticatedUser,
    owner: StaffId,
) -> Result<bool, UseCaseError> {
    if user.has_role(Role::Admin) {
        return Ok(true);
    }
    let me = staff_repository.find_by_user_id(&user.user_id).await?;
    Ok(me.is_some_and(|s| s.id() == owner))
}

/// 作成中の給与明細は、管理者が内容を確かめている段階なので、派遣社員本人には見せない
fn is_visible_to(user: &AuthenticatedUser, payslip: &Payslip) -> bool {
    user.has_role(Role::Admin) || payslip.status() == PayslipStatus::Finalized
}

/// 給与明細を1件見る。管理者はすべて、派遣社員は自分の確定済みのものだけ見られる
pub struct GetPayslipUseCase {
    repository: Arc<dyn PayslipRepository>,
    staff_repository: Arc<dyn StaffRepository>,
}

impl GetPayslipUseCase {
    #[must_use]
    pub fn new(
        repository: Arc<dyn PayslipRepository>,
        staff_repository: Arc<dyn StaffRepository>,
    ) -> Self {
        Self { repository, staff_repository }
    }

    /// 給与明細番号で給与明細を返す。
    ///
    /// 見る権限のない給与明細は、存在しないものと同じく `NotFound` になる。
    /// 他人の給与明細や作成中の給与明細が存在するかどうかも、見る権限のない人には知らせない
    pub async fn execute(
        &self,
        user: &AuthenticatedUser,
        id: PayslipId,
    ) -> Result<Payslip, UseCaseError> {
        let payslip = self.repository.find(id).await?.ok_or(UseCaseError::NotFound)?;

        if !can_view(self.staff_repository.as_ref(), user, payslip.content().staff_id()).await?
            || !is_visible_to(user, &payslip)
        {
            return Err(UseCaseError::NotFound);
        }
        Ok(payslip)
    }
}

/// 派遣社員の給与明細を一覧する。管理者は誰のものでも、派遣社員は自分の確定済みのものだけ見られる
pub struct ListPayslipsUseCase {
    repository: Arc<dyn PayslipRepository>,
    staff_repository: Arc<dyn StaffRepository>,
}

impl ListPayslipsUseCase {
    #[must_use]
    pub fn new(
        repository: Arc<dyn PayslipRepository>,
        staff_repository: Arc<dyn StaffRepository>,
    ) -> Self {
        Self { repository, staff_repository }
    }

    /// 派遣社員の有効な給与明細を、新しい月から順に返す。
    /// 見る権限がなければ `NotFound` になる
    pub async fn execute(
        &self,
        user: &AuthenticatedUser,
        staff_id: StaffId,
    ) -> Result<Vec<Payslip>, UseCaseError> {
        if !can_view(self.staff_repository.as_ref(), user, staff_id).await? {
            return Err(UseCaseError::NotFound);
        }
        let payslips = self.repository.list_by_staff(staff_id).await?;
        Ok(payslips.into_iter().filter(|p| is_visible_to(user, p)).collect())
    }
}
