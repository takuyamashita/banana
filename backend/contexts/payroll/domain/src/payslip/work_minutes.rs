use super::PayslipError;

/// 1つの案件での、1か月分の稼働時間(分)。
///
/// 稼働は15分単位で数え、15分に満たない端数は切り捨てる(100分の稼働は90分として扱う)。
/// 切り捨てた結果が0分になるもの(14分以下)は稼働として認めない。
/// 1か月の上限は 744時間(31日 × 24時間)で、これを超える稼働はありえないので受け付けない
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkMinutes(u32);

/// 1か月の稼働時間の上限(分)。31日 × 24時間
const MAX_WORK_MINUTES: u32 = 744 * 60;

impl WorkMinutes {
    pub fn from_minutes(minutes: u32) -> Result<Self, PayslipError> {
        let floored = minutes - minutes % 15;
        if floored == 0 || minutes > MAX_WORK_MINUTES {
            return Err(PayslipError::InvalidWorkMinutes);
        }
        Ok(Self(floored))
    }

    /// 15分単位に切り捨てた後の分数
    #[must_use]
    pub fn as_minutes(&self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floored_to_15_minutes() {
        assert_eq!(WorkMinutes::from_minutes(100).unwrap().as_minutes(), 90);
    }

    #[test]
    fn less_than_15_minutes_is_rejected() {
        assert_eq!(WorkMinutes::from_minutes(14), Err(PayslipError::InvalidWorkMinutes));
        assert_eq!(WorkMinutes::from_minutes(0), Err(PayslipError::InvalidWorkMinutes));
    }

    #[test]
    fn over_a_month_is_rejected() {
        assert_eq!(WorkMinutes::from_minutes(744 * 60 + 1), Err(PayslipError::InvalidWorkMinutes));
    }
}
