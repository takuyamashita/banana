use super::PayslipError;

/// 時給(円)。案件ごとに決まり、同じ派遣社員でも案件によって異なる。
///
/// 1円以上、10万円以下。それを超える時給は入力の誤りとみなす
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HourlyRate(u32);

impl HourlyRate {
    /// 時給の上限(円)
    pub const MAX_YEN: u32 = 100_000;

    pub fn from_yen(yen: i64) -> Result<Self, PayslipError> {
        u32::try_from(yen)
            .ok()
            .filter(|y| (1..=Self::MAX_YEN).contains(y))
            .map(Self)
            .ok_or(PayslipError::InvalidHourlyRate { max: Self::MAX_YEN })
    }

    #[must_use]
    pub fn as_yen(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_one_yen_up_to_the_limit() {
        assert_eq!(HourlyRate::from_yen(1).unwrap().as_yen(), 1);
        assert_eq!(HourlyRate::from_yen(100_000).unwrap().as_yen(), 100_000);
    }

    #[test]
    fn zero_negative_and_over_the_limit_are_rejected() {
        for yen in [0, -1, 100_001, i64::MAX] {
            assert_eq!(
                HourlyRate::from_yen(yen),
                Err(PayslipError::InvalidHourlyRate { max: 100_000 }),
                "{yen}"
            );
        }
    }
}
