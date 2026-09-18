//! どのレイヤーからも使う最小限の共通値。
//!
//! `AuthenticatedUser` はここに置く。`shared/auth` はトークン検証のために HTTP・JWT 系の
//! crate に依存するので、usecase がそちらに依存すると I/O 系 crate を引き込んでしまうため。

use std::fmt;
use std::ops::{Add, Mul};

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MoneyError {
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

    /// 切り捨て除算。端数の扱いは呼び出し側(ドメイン)の業務ルールで選ぶ
    #[must_use]
    pub fn div_floor(self, divisor: i64) -> Self {
        Self(self.0.div_euclid(divisor))
    }
}

impl Add for Money {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl Mul<u32> for Money {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self {
        Self(self.0.saturating_mul(i64::from(rhs)))
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}円", self.0)
    }
}

/// アプリが扱うロール。プロバイダ固有のクレーム(`cognito:groups`・`realm_access.roles`)は
/// shared/auth の mapper でこれに詰め替える
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    Staff,
}

impl Role {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "admin" => Some(Self::Admin),
            "staff" => Some(Self::Staff),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Staff => "staff",
        }
    }
}

/// 検証済みトークンから作った利用者。`user_id` は認証基盤上のID(sub)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub roles: Vec<Role>,
}

impl AuthenticatedUser {
    #[must_use]
    pub fn has_role(&self, role: Role) -> bool {
        self.roles.contains(&role)
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
    fn div_floor_truncates() {
        let m = Money::from_yen(1_999).unwrap();
        assert_eq!(m.div_floor(60).as_yen(), 33);
    }
}
