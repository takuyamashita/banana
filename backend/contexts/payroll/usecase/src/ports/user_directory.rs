use async_trait::async_trait;
use platform_kernel::{Email, UserId};
use thiserror::Error;

/// ログイン用アカウントを発行・削除できなかった理由
#[derive(Debug, Error)]
pub enum UserDirectoryError {
    /// 同じメールアドレスのアカウントが既にある
    #[error("同じメールアドレスの利用者が既にいます")]
    AlreadyExists,
    /// 仮パスワードが認証基盤の規則(長さ・文字の種類)に合わない
    #[error("仮パスワードが規則に合いません(12文字以上で、大文字・小文字・数字を含めてください)")]
    InvalidPassword,
    /// メールアドレスなどの入力が受け付けられなかった。理由は認証基盤の言葉なので、利用者には見せない
    #[error("入力が受け付けられませんでした")]
    Invalid { detail: String },
    /// アカウントの発行元に一時的につながらない、または想定外の応答が返った
    #[error("認証基盤が利用できません: {0}")]
    Unavailable(String),
}

/// 派遣社員のログイン用アカウントを発行・削除する先
#[async_trait]
pub trait UserDirectory: Send + Sync {
    /// メールアドレスでログインする派遣社員のアカウントを発行し、その利用者IDを返す。
    /// アカウントには派遣社員のロールが付く。仮パスワードは初回ログインで本人が変更する
    async fn create_user(
        &self,
        email: &Email,
        temporary_password: &str,
    ) -> Result<UserId, UserDirectoryError>;

    /// アカウントを消す。派遣社員の登録に失敗したときに、発行したアカウントを残さないために使う
    /// (停止だけでは、同じメールアドレスで登録し直せなくなる)
    async fn delete_user(&self, id: &UserId) -> Result<(), UserDirectoryError>;
}
