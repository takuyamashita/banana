//! 派遣社員(staff)。
//!
//! 勤務表を書く人。給与(payroll)で登録された派遣社員の写しで、勤怠では登録も変更もしない。
//! 派遣社員は自分の勤務表を書くためにログインするので、ログイン用のアカウント(利用者)で見分ける

use platform_kernel::UserId;
use thiserror::Error;

platform_kernel::positive_id! {
    /// 派遣社員番号。給与(payroll)で振られた番号をそのまま使う
    pub struct StaffId;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StaffError {
    #[error("派遣社員の名前は1〜100文字で入力してください")]
    InvalidName,
}

/// 派遣社員の名前(管理者が勤務表を見分けるのに使う)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffName(String);

impl StaffName {
    pub fn new(name: impl Into<String>) -> Result<Self, StaffError> {
        let name = name.into();
        let length = name.trim().chars().count();
        if length == 0 || length > 100 {
            return Err(StaffError::InvalidName);
        }
        Ok(Self(name))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 勤務表を書く派遣社員
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staff {
    /// 派遣社員番号
    id: StaffId,
    /// この派遣社員のログイン用アカウント
    user_id: UserId,
    /// 名前
    name: StaffName,
}

impl Staff {
    #[must_use]
    pub fn new(id: StaffId, user_id: UserId, name: StaffName) -> Self {
        Self { id, user_id, name }
    }

    #[must_use]
    pub fn id(&self) -> StaffId {
        self.id
    }

    #[must_use]
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    #[must_use]
    pub fn name(&self) -> &StaffName {
        &self.name
    }
}
