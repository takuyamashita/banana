use time::macros::offset;
use time::{Date, Month, OffsetDateTime, UtcOffset};

use super::PayslipError;

/// 給与の対象月(「2026年9月分」)。
///
/// 月の区切りは日本時間で数える。9月分は 9月1日 0:00(JST)から 10月1日 0:00(JST)の直前までに
/// 行われた稼働が対象になる。扱う年は 2000年から 2999年まで
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PayPeriod {
    /// 西暦年
    year: u16,
    /// 月(1〜12)
    month: u8,
    /// この月に含まれる時刻の始まり(月初 0:00 JST)
    start: OffsetDateTime,
    /// この月に含まれる時刻の終わり(翌月初 0:00 JST)。この時刻は含まない
    end: OffsetDateTime,
}

/// 日本時間。日本にはサマータイムがないので、常に UTC+9
const JST: UtcOffset = offset!(+9);

impl PayPeriod {
    pub fn new(year: u16, month: u8) -> Result<Self, PayslipError> {
        if !(2000..=2999).contains(&year) {
            return Err(PayslipError::InvalidPeriod);
        }
        let month_of_year = Month::try_from(month).map_err(|_| PayslipError::InvalidPeriod)?;
        let (next_year, next_month) = if month_of_year == Month::December {
            (year + 1, Month::January)
        } else {
            (year, month_of_year.next())
        };
        Ok(Self {
            year,
            month,
            start: first_moment_jst(year, month_of_year)?,
            end: first_moment_jst(next_year, next_month)?,
        })
    }

    #[must_use]
    pub fn year(&self) -> u16 {
        self.year
    }

    #[must_use]
    pub fn month(&self) -> u8 {
        self.month
    }

    /// この月に含まれる時刻の範囲を UTC で返す。始まりを含み、終わり(翌月初 0:00 JST)を含まない
    #[must_use]
    pub fn range_utc(&self) -> (OffsetDateTime, OffsetDateTime) {
        (self.start.to_offset(UtcOffset::UTC), self.end.to_offset(UtcOffset::UTC))
    }
}

/// 月初 0:00(JST)
fn first_moment_jst(year: u16, month: Month) -> Result<OffsetDateTime, PayslipError> {
    Ok(Date::from_calendar_date(i32::from(year), month, 1)
        .map_err(|_| PayslipError::InvalidPeriod)?
        .midnight()
        .assume_offset(JST))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_out_of_1_to_12_is_rejected() {
        assert_eq!(PayPeriod::new(2026, 0), Err(PayslipError::InvalidPeriod));
        assert_eq!(PayPeriod::new(2026, 13), Err(PayslipError::InvalidPeriod));
    }

    #[test]
    fn year_out_of_range_is_rejected() {
        assert_eq!(PayPeriod::new(1999, 12), Err(PayslipError::InvalidPeriod));
        assert_eq!(PayPeriod::new(3000, 1), Err(PayslipError::InvalidPeriod));
        assert!(PayPeriod::new(2999, 12).is_ok());
    }

    #[test]
    fn range_is_half_open_in_jst() {
        let (start, end) = PayPeriod::new(2026, 12).unwrap().range_utc();
        assert_eq!(start.to_string(), "2026-11-30 15:00:00.0 +00:00:00");
        assert_eq!(end.to_string(), "2026-12-31 15:00:00.0 +00:00:00");
    }
}
