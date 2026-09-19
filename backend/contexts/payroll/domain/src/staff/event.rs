use platform_kernel::UserId;

use super::{DisplayName, StaffId};

/// 派遣社員に起きた、他の業務が知るべき出来事
#[must_use = "派遣社員の出来事は記録して他の業務に知らせる必要がある"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaffEvent {
    /// 派遣社員が登録された。他の業務(勤怠など)は、これで派遣社員と、その人のログイン用アカウントを結びつける
    Registered {
        /// 振られた派遣社員番号
        staff_id: StaffId,
        /// ログイン用アカウント
        user_id: UserId,
        /// 表示名
        display_name: DisplayName,
    },
}
