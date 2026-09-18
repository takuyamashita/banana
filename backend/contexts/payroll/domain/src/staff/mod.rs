//! 派遣社員(staff)。
//!
//! 派遣会社に登録し、派遣先の案件で働く人の雇用記録。給与明細はこの派遣社員ごとに作る。
//! 派遣社員は自分の給与明細を見るためにログインするので、ログイン用のアカウント(利用者)と
//! 1対1で結びつく。雇用記録を指す派遣社員番号と、アカウントを指す利用者IDは別のもの。

use platform_kernel::Unsaved;
use thiserror::Error;

/// 派遣社員の業務ルールに反したときの理由
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StaffError {
    /// 派遣社員番号が正の数でない
    #[error("派遣社員IDが不正です")]
    InvalidId,
    /// 利用者IDが空、または長すぎる
    #[error("利用者IDが不正です")]
    InvalidUserId,
    /// メールアドレスの形になっていない
    #[error("メールアドレスが不正です")]
    InvalidEmail,
    /// 表示名が空、または50文字を超えている
    #[error("表示名は1〜50文字で指定してください")]
    InvalidDisplayName,
}

/// 派遣社員番号。雇用記録を一意に指す正の整数。給与明細はこの番号で派遣社員を指す
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StaffId(i64);

impl StaffId {
    pub fn from_i64(value: i64) -> Result<Self, StaffError> {
        if value <= 0 {
            return Err(StaffError::InvalidId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

/// 利用者ID。派遣社員がログインに使うアカウントを指す識別子。
///
/// アカウントは認証基盤が発行し、形式はそちらが決める(UUID など)。ここでは空でなく、
/// 64文字以内であることだけを求める
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn parse(value: impl Into<String>) -> Result<Self, StaffError> {
        let value = value.into();
        if value.is_empty() || value.len() > 64 {
            return Err(StaffError::InvalidUserId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// メールアドレス。ログインと連絡に使う。
///
/// 前後の空白を除き、小文字にそろえて扱う(大文字小文字の違いで別人とみなさない)。
/// 同じメールアドレスの派遣社員は2人登録できない
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(value: impl Into<String>) -> Result<Self, StaffError> {
        let value = value.into().trim().to_ascii_lowercase();
        let valid = value.len() <= 254
            && value.split_once('@').is_some_and(|(local, domain)| {
                !local.is_empty() && domain.contains('.') && !domain.starts_with('.')
            });
        if !valid {
            return Err(StaffError::InvalidEmail);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 表示名。画面で派遣社員を見分けるための名前(氏名など)で、前後の空白を除いて1〜50文字
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayName(String);

impl DisplayName {
    pub fn new(value: impl Into<String>) -> Result<Self, StaffError> {
        let value = value.into().trim().to_owned();
        if value.is_empty() || value.chars().count() > 50 {
            return Err(StaffError::InvalidDisplayName);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 派遣社員。派遣先の案件で働く人の雇用記録と、その人のログイン用アカウントの対応。
///
/// `Id` はまだ登録していない派遣社員なら [`Unsaved`]、登録済みなら [`StaffId`]
#[derive(Debug)]
pub struct Staff<Id = StaffId> {
    /// 派遣社員番号
    id: Id,
    /// この派遣社員のログイン用アカウント。1人に1つ
    user_id: UserId,
    /// メールアドレス
    email: Email,
    /// 表示名
    display_name: DisplayName,
}

/// まだ登録していない派遣社員
pub type NewStaff = Staff<Unsaved>;

impl<Id> Staff<Id> {
    #[must_use]
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    #[must_use]
    pub fn email(&self) -> &Email {
        &self.email
    }

    #[must_use]
    pub fn display_name(&self) -> &DisplayName {
        &self.display_name
    }
}

impl Staff<Unsaved> {
    /// 派遣社員を新しく作る。ログイン用アカウントは先に用意しておく
    #[must_use]
    pub fn new(user_id: UserId, email: Email, display_name: DisplayName) -> Self {
        Self { id: Unsaved, user_id, email, display_name }
    }
}

impl Staff<StaffId> {
    /// 登録済みの派遣社員を、記録されている内容から組み立て直す
    #[must_use]
    pub fn reconstruct(
        id: StaffId,
        user_id: UserId,
        email: Email,
        display_name: DisplayName,
    ) -> Self {
        Self { id, user_id, email, display_name }
    }

    #[must_use]
    pub fn id(&self) -> StaffId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_is_normalized() {
        assert_eq!(Email::parse(" Foo@Example.COM ").unwrap().as_str(), "foo@example.com");
    }

    #[test]
    fn email_without_domain_is_rejected() {
        assert_eq!(Email::parse("foo@"), Err(StaffError::InvalidEmail));
        assert_eq!(Email::parse("foo@localhost"), Err(StaffError::InvalidEmail));
    }
}
