use platform_kernel::{Email, Unsaved, UserId};

use super::{DisplayName, StaffId};

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
