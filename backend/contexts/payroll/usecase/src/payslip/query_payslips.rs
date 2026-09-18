use std::sync::Arc;

use payroll_domain::payslip::{Payslip, PayslipId, PayslipRepository};
use payroll_domain::staff::{StaffId, StaffRepository, UserId};
use platform_kernel::{AuthenticatedUser, Role};

use crate::UseCaseError;

/// 明細を見てよいのは管理者か、明細の本人だけ。
/// ID は連番で API に露出するので、この確認が他人の明細を引けないことを保証する唯一の手段
async fn can_view(
    staff_repository: &dyn StaffRepository,
    user: &AuthenticatedUser,
    owner: StaffId,
) -> Result<bool, UseCaseError> {
    if user.has_role(Role::Admin) {
        return Ok(true);
    }
    let user_id = UserId::parse(user.user_id.clone())?;
    let me = staff_repository.find_by_user_id(&user_id).await?;
    Ok(me.is_some_and(|s| s.id() == owner))
}

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

    pub async fn execute(
        &self,
        user: &AuthenticatedUser,
        id: PayslipId,
    ) -> Result<Payslip, UseCaseError> {
        let payslip = self.repository.find(id).await?.ok_or(UseCaseError::NotFound)?;

        // 他人の明細は「存在しない」と同じ応答にし、IDの存在を漏らさない
        if !can_view(self.staff_repository.as_ref(), user, payslip.staff_id()).await? {
            return Err(UseCaseError::NotFound);
        }
        Ok(payslip)
    }
}

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

    pub async fn execute(
        &self,
        user: &AuthenticatedUser,
        staff_id: StaffId,
    ) -> Result<Vec<Payslip>, UseCaseError> {
        if !can_view(self.staff_repository.as_ref(), user, staff_id).await? {
            return Err(UseCaseError::NotFound);
        }
        Ok(self.repository.list_by_staff(staff_id).await?)
    }
}
