//! 管理者のログイン用アカウントの発行

use std::sync::Arc;

use platform_kernel::{Email, Role, UserId};

use crate::UseCaseError;
use crate::ports::user_directory::UserDirectory;

/// 管理者の発行で入力する内容
pub struct CreateAdminUserInput {
    /// ログインに使うメールアドレス
    pub email: Email,
    /// 初回ログイン用の仮パスワード。本人が初回ログインで変更する
    pub temporary_password: String,
}

/// 管理者が、もう1人の管理者を作る。
///
/// 管理者は雇用の記録を持たない(給与明細も振込もない)ので、作るのは認証基盤のアカウントだけで、
/// こちらの記録は増えない。派遣社員は雇用の記録も要るので [`crate::staff::CreateStaffUseCase`] を使う
pub struct CreateAdminUserUseCase {
    user_directory: Arc<dyn UserDirectory>,
}

impl CreateAdminUserUseCase {
    #[must_use]
    pub fn new(user_directory: Arc<dyn UserDirectory>) -> Self {
        Self { user_directory }
    }

    /// 管理者のアカウントを発行し、認証基盤が振った利用者IDを返す。
    ///
    /// そのメールアドレスのアカウントが既にあれば `Conflict` になる。派遣社員もアカウントを
    /// メールアドレスで持つので、派遣社員と同じメールアドレスの管理者も作れない
    pub async fn execute(&self, input: CreateAdminUserInput) -> Result<UserId, UseCaseError> {
        Ok(self
            .user_directory
            .create_user(&input.email, &input.temporary_password, Role::Admin)
            .await?)
    }
}
