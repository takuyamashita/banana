use async_trait::async_trait;
use payroll_domain::staff::{Email, UserId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserDirectoryError {
    #[error("同じメールアドレスの利用者が既にいます")]
    AlreadyExists,
    #[error("認証基盤が入力を受け付けませんでした: {0}")]
    Invalid(String),
    #[error("認証基盤が利用できません: {0}")]
    Unavailable(String),
}

// プロバイダの語彙(user pool、realmなど)はここに出さない
#[async_trait]
pub trait UserDirectory: Send + Sync {
    async fn create_user(
        &self,
        email: &Email,
        temporary_password: &str,
    ) -> Result<UserId, UserDirectoryError>;

    async fn disable_user(&self, id: &UserId) -> Result<(), UserDirectoryError>;
}
