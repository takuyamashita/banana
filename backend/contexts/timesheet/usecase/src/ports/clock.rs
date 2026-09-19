//! 今の日時

use time::OffsetDateTime;

/// 今の日時を知る先。勤務表を申告・承認した日時に使う
pub trait Clock: Send + Sync {
    fn now(&self) -> OffsetDateTime;
}
