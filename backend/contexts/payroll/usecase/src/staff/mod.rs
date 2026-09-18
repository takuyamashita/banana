use std::sync::Arc;

use payroll_domain::staff::{DisplayName, Email, NewStaff, Staff, StaffId, UserId};
use platform_kernel::AuthenticatedUser;

use crate::UseCaseError;
use crate::ports::queries::{StaffQuery, StaffView};
use crate::ports::repository::StaffRepository;
use crate::ports::user_directory::UserDirectory;

pub struct CreateStaffInput {
    pub email: Email,
    pub display_name: DisplayName,
    pub temporary_password: String,
}

/// 派遣社員を登録する。認証基盤にユーザーを作り、その ID と雇用記録を対応付ける
pub struct CreateStaffUseCase {
    staff_repository: Arc<dyn StaffRepository>,
    user_directory: Arc<dyn UserDirectory>,
}

impl CreateStaffUseCase {
    #[must_use]
    pub fn new(
        staff_repository: Arc<dyn StaffRepository>,
        user_directory: Arc<dyn UserDirectory>,
    ) -> Self {
        Self { staff_repository, user_directory }
    }

    pub async fn execute(&self, input: CreateStaffInput) -> Result<StaffId, UseCaseError> {
        if self.staff_repository.find_by_email(&input.email).await?.is_some() {
            return Err(UseCaseError::Conflict("同じメールアドレスの派遣社員がいます".into()));
        }

        let user_id =
            self.user_directory.create_user(&input.email, &input.temporary_password).await?;

        let new = NewStaff::new(user_id.clone(), input.email, input.display_name);
        match self.staff_repository.insert(&new).await {
            Ok(id) => Ok(id),
            Err(err) => {
                // DB と認証基盤はトランザクションを共有できないので、失敗したら作ったユーザーを無効化して戻す。
                // 無効化にも失敗した場合は元のエラーを優先して返す(孤立ユーザーは運用で掃除する)
                let _ = self.user_directory.disable_user(&user_id).await;
                Err(err.into())
            }
        }
    }
}

pub struct ListStaffUseCase {
    query: Arc<dyn StaffQuery>,
}

impl ListStaffUseCase {
    #[must_use]
    pub fn new(query: Arc<dyn StaffQuery>) -> Self {
        Self { query }
    }

    pub async fn execute(&self) -> Result<Vec<StaffView>, UseCaseError> {
        Ok(self.query.list().await?)
    }
}

/// ログイン中の利用者に対応する派遣社員。管理者など未登録なら None
pub struct GetMeUseCase {
    staff_repository: Arc<dyn StaffRepository>,
}

impl GetMeUseCase {
    #[must_use]
    pub fn new(staff_repository: Arc<dyn StaffRepository>) -> Self {
        Self { staff_repository }
    }

    pub async fn execute(&self, user: &AuthenticatedUser) -> Result<Option<Staff>, UseCaseError> {
        let user_id = UserId::parse(user.user_id.clone())?;
        Ok(self.staff_repository.find_by_user_id(&user_id).await?)
    }
}
