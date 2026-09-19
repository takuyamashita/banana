use time::{Date, Month};

use super::TimesheetError;

/// 勤務表の対象月(「2026年9月」)。日付はその土地の暦の日付(時刻を持たない)で数える。
/// 扱う年は 2000年から 2999年まで
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkMonth {
    year: u16,
    month: Month,
}

impl WorkMonth {
    pub fn new(year: u16, month: u8) -> Result<Self, TimesheetError> {
        if !(2000..=2999).contains(&year) {
            return Err(TimesheetError::InvalidMonth);
        }
        let month = Month::try_from(month).map_err(|_| TimesheetError::InvalidMonth)?;
        Ok(Self { year, month })
    }

    #[must_use]
    pub fn year(&self) -> u16 {
        self.year
    }

    #[must_use]
    pub fn month(&self) -> u8 {
        u8::from(self.month)
    }

    /// その日がこの月の日か
    #[must_use]
    pub fn contains(&self, date: Date) -> bool {
        i32::from(self.year) == date.year() && self.month == date.month()
    }
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    #[test]
    fn months_outside_the_supported_range_are_rejected() {
        assert_eq!(WorkMonth::new(1999, 12), Err(TimesheetError::InvalidMonth));
        assert_eq!(WorkMonth::new(2026, 13), Err(TimesheetError::InvalidMonth));
        assert_eq!(WorkMonth::new(2026, 0), Err(TimesheetError::InvalidMonth));
    }

    #[test]
    fn a_month_contains_only_its_own_days() {
        let september = WorkMonth::new(2026, 9).unwrap();
        assert!(september.contains(date!(2026 - 09 - 01)));
        assert!(september.contains(date!(2026 - 09 - 30)));
        assert!(!september.contains(date!(2026 - 10 - 01)));
        assert!(!september.contains(date!(2025 - 09 - 15)));
    }
}
