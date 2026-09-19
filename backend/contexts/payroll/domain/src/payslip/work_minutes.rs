use super::PayslipError;

/// 1つの案件での、1か月分の稼働時間(分)。
///
/// 稼働は15分単位で数える。15分の倍数でない時間は受け付けない
/// (切り捨てると、働いた時間の賃金が払われなくなる)。
/// 1か月の上限は 744時間(31日 × 24時間)で、これを超える稼働はありえないので受け付けない
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkMinutes(u32);

/// 1か月の稼働時間の上限(分)。31日 × 24時間
pub(super) const MAX_WORK_MINUTES: u32 = 744 * 60;

impl WorkMinutes {
    pub fn from_minutes(minutes: u32) -> Result<Self, PayslipError> {
        if minutes == 0 || !minutes.is_multiple_of(15) || minutes > MAX_WORK_MINUTES {
            return Err(PayslipError::InvalidWorkMinutes);
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
    fn multiples_of_15_minutes_up_to_a_month() {
        assert_eq!(WorkMinutes::from_minutes(15).unwrap().as_minutes(), 15);
        assert_eq!(WorkMinutes::from_minutes(744 * 60).unwrap().as_minutes(), 744 * 60);
    }

    #[test]
    fn not_a_multiple_of_15_minutes_is_rejected_not_rounded() {
        assert_eq!(WorkMinutes::from_minutes(100), Err(PayslipError::InvalidWorkMinutes));
        assert_eq!(WorkMinutes::from_minutes(14), Err(PayslipError::InvalidWorkMinutes));
    }

    #[test]
    fn zero_and_over_a_month_are_rejected() {
        assert_eq!(WorkMinutes::from_minutes(0), Err(PayslipError::InvalidWorkMinutes));
        assert_eq!(WorkMinutes::from_minutes(744 * 60 + 15), Err(PayslipError::InvalidWorkMinutes));
    }
}
