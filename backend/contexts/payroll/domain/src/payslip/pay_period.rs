use time::{Date, Month, OffsetDateTime, UtcOffset};

use super::PayslipError;

/// 給与の対象月(「2026年9月分」)。
///
/// 月の区切りは日本時間で数える。9月分は 9月1日 0:00(JST)から 10月1日 0:00(JST)の直前までに
/// 行われた稼働が対象になる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PayPeriod {
    /// 西暦年
    year: u16,
    /// 月(1〜12)
    month: u8,
}

impl PayPeriod {
    pub fn new(year: u16, month: u8) -> Result<Self, PayslipError> {
        if !(1..=12).contains(&month) {
            return Err(PayslipError::InvalidPeriod);
        }
        Ok(Self { year, month })
    }

    #[must_use]
    pub fn year(&self) -> u16 {
        self.year
    }

    #[must_use]
    pub fn month(&self) -> u8 {
        self.month
    }

    /// この月に含まれる時刻の範囲を UTC で返す。
    ///
    /// 始まり(月初 0:00 JST)を含み、終わり(翌月初 0:00 JST)を含まない。
    /// 「月末 23:59:59 まで」とすると、その1秒の間の時刻が漏れるため、終わりは翌月初で表す
    #[must_use]
    pub fn range_utc(&self) -> (OffsetDateTime, OffsetDateTime) {
        let start = self.first_moment_jst();
        let end = self.next().first_moment_jst();
        (start.to_offset(UtcOffset::UTC), end.to_offset(UtcOffset::UTC))
    }

    /// 月初 0:00(JST)。日本にはサマータイムがないので、常に UTC+9 で数える
    fn first_moment_jst(self) -> OffsetDateTime {
        let jst = UtcOffset::from_hms(9, 0, 0).expect("JST");
        Date::from_calendar_date(
            i32::from(self.year),
            Month::try_from(self.month).expect("1..=12"),
            1,
        )
        .expect("valid date")
        .midnight()
        .assume_offset(jst)
    }

    fn next(self) -> Self {
        if self.month == 12 {
            Self { year: self.year + 1, month: 1 }
        } else {
            Self { year: self.year, month: self.month + 1 }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn month_13_is_rejected() {
        assert_eq!(PayPeriod::new(2026, 13), Err(PayslipError::InvalidPeriod));
    }

    #[test]
    fn range_is_half_open_in_jst() {
        let (start, end) = PayPeriod::new(2026, 12).unwrap().range_utc();
        assert_eq!(start.to_string(), "2026-11-30 15:00:00.0 +00:00:00");
        assert_eq!(end.to_string(), "2026-12-31 15:00:00.0 +00:00:00");
    }
}
