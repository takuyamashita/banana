use super::TimesheetError;

/// 稼働を数える単位(分)。勤怠は15分単位で数える
const UNIT: u32 = 15;
/// 1日の稼働の上限(分)
pub(super) const MINUTES_PER_DAY: u32 = 24 * 60;

/// 1日の、1つの案件での稼働(分)。15分単位で、15分から24時間まで
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkMinutes(u32);

impl WorkMinutes {
    /// 15分単位でない稼働は丸めずに断る(切り捨てると、働いた分が数えられない)
    pub fn from_minutes(minutes: u32) -> Result<Self, TimesheetError> {
        if minutes == 0 || minutes > MINUTES_PER_DAY || !minutes.is_multiple_of(UNIT) {
            return Err(TimesheetError::InvalidWorkMinutes);
        }
        Ok(Self(minutes))
    }

    #[must_use]
    pub fn as_minutes(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_is_counted_in_units_of_fifteen_minutes_up_to_a_day() {
        assert_eq!(WorkMinutes::from_minutes(15).unwrap().as_minutes(), 15);
        assert_eq!(WorkMinutes::from_minutes(1440).unwrap().as_minutes(), 1440);
        for minutes in [0, 10, 470, 1455] {
            assert_eq!(WorkMinutes::from_minutes(minutes), Err(TimesheetError::InvalidWorkMinutes));
        }
    }
}
