//! 今の日時(usecase の ports::clock)の実装

use payroll_usecase::ports::clock::Clock;
use time::OffsetDateTime;

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
