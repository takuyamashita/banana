use std::fmt;

use thiserror::Error;

/// 金額として成り立たない理由
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MoneyError {
    /// マイナスの金額
    #[error("金額は0円以上である必要があります")]
    Negative,
}

/// 円建ての金額。円未満は扱わない
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Self = Self(0);

    pub fn from_yen(yen: i64) -> Result<Self, MoneyError> {
        if yen < 0 {
            return Err(MoneyError::Negative);
        }
        Ok(Self(yen))
    }

    #[must_use]
    pub fn as_yen(&self) -> i64 {
        self.0
    }

    /// 足し算。桁があふれるときは `None`
    #[must_use]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}円", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_yen_is_rejected() {
        assert_eq!(Money::from_yen(-1), Err(MoneyError::Negative));
    }

    #[test]
    fn overflowing_addition_is_none() {
        let max = Money::from_yen(i64::MAX).unwrap();
        assert_eq!(max.checked_add(Money::from_yen(1).unwrap()), None);
        assert_eq!(
            Money::from_yen(1).unwrap().checked_add(Money::from_yen(2).unwrap()),
            Some(Money::from_yen(3).unwrap())
        );
    }
}
