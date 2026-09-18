use super::PayslipError;

/// 給与明細番号。登録された給与明細を一意に指す正の整数
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PayslipId(i64);

impl PayslipId {
    pub fn from_i64(value: i64) -> Result<Self, PayslipError> {
        if value <= 0 {
            return Err(PayslipError::InvalidId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}
