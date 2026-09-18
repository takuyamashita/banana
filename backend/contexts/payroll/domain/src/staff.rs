//! 派遣社員(staff)。
//!
//! 派遣会社に登録し、派遣先の案件で働く人の雇用記録。給与明細はこの派遣社員ごとに作る。
//! 派遣社員は自分の給与明細を見るためにログインするので、ログイン用のアカウント(利用者)と
//! 1対1で結びつく。雇用記録を指す派遣社員番号と、アカウントを指す利用者IDは別のもの。

use platform_kernel::{Email, Unsaved, UserId};
use thiserror::Error;

/// 派遣社員の業務ルールに反したときの理由
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StaffError {
    /// 表示名が空、または50文字を超えている
    #[error("表示名は1〜50文字で指定してください")]
    InvalidDisplayName,
}

platform_kernel::positive_id! {
    /// 派遣社員番号。雇用記録を一意に指す正の整数。給与明細はこの番号で派遣社員を指す
    pub struct StaffId;
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

/// 派遣社員。派遣先の案件で働く人の雇用記録と、その人のログイン用アカウントの対応
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
