//! 今の日時(usecase の ports::clock)の実装

use time::OffsetDateTime;
use timesheet_usecase::ports::clock::Clock;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
