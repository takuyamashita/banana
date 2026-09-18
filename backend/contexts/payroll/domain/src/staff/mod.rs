//! 派遣社員(staff)集約。
//!
//! `UserId` は認証基盤上のID(Cognito の sub)で、雇用記録を指す `StaffId` とは別物。
//! 両者の対応付けはこの集約が持つ。

use thiserror::Error;

use crate::Unsaved;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StaffError {
    #[error("派遣社員IDが不正です")]
    InvalidId,
    #[error("利用者IDが不正です")]
    InvalidUserId,
    #[error("メールアドレスが不正です")]
    InvalidEmail,
    #[error("表示名は1〜50文字で指定してください")]
    InvalidDisplayName,
}

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

/// 認証基盤上の利用者ID。形式はプロバイダ次第なので、空でないことと長さだけを見る
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

/// 派遣社員。`Id` は保存済みなら `StaffId`、未保存なら `Unsaved`
#[derive(Debug)]
pub struct Staff<Id = StaffId> {
    id: Id,
    user_id: UserId,
    email: Email,
    display_name: DisplayName,
}

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
    #[must_use]
    pub fn new(user_id: UserId, email: Email, display_name: DisplayName) -> Self {
        Self { id: Unsaved, user_id, email, display_name }
    }
}

impl Staff<StaffId> {
    // 永続化からの再構築専用
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
