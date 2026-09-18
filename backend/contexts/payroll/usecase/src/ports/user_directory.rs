use async_trait::async_trait;
use payroll_domain::staff::{Email, UserId};
use thiserror::Error;

/// ログイン用アカウントを発行・停止できなかった理由
#[derive(Debug, Error)]
pub enum UserDirectoryError {
    /// 同じメールアドレスのアカウントが既にある
    #[error("同じメールアドレスの利用者が既にいます")]
    AlreadyExists,
    /// メールアドレスや仮パスワードが受け付けられなかった(パスワードが短すぎるなど)
    #[error("認証基盤が入力を受け付けませんでした: {0}")]
    Invalid(String),
    /// アカウントの発行元に一時的につながらない
    #[error("認証基盤が利用できません: {0}")]
    Unavailable(String),
}

/// 派遣社員のログイン用アカウントを発行・停止する先
#[async_trait]
pub trait UserDirectory: Send + Sync {
    /// メールアドレスでログインするアカウントを発行し、その利用者IDを返す。
    /// 仮パスワードは初回ログインで本人が変更する
    async fn create_user(
        &self,
        email: &Email,
        temporary_password: &str,
    ) -> Result<UserId, UserDirectoryError>;

    /// アカウントを使えなくする(ログインできなくなる)
    async fn disable_user(&self, id: &UserId) -> Result<(), UserDirectoryError>;
}
