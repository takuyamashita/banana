//! 今の日時

use time::OffsetDateTime;

/// 今の日時を知る先。給与を確定した日時などに使う
pub trait Clock: Send + Sync {
    fn now(&self) -> OffsetDateTime;
}
