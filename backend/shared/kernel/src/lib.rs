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

/// 番号(ID)が正の整数でない
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("番号は正の整数である必要があります")]
pub struct InvalidId;

/// 正の整数の番号(ID)の型を宣言する。
///
/// 宣言した型ごとに別の型になるので、給与明細番号と派遣社員番号の取り違えはコンパイルエラーになる。
/// `from_i64`(0 以下は [`InvalidId`])と `as_i64` を持つ。
///
/// ```
/// platform_kernel::positive_id! {
///     /// 給与明細番号
///     pub struct PayslipId;
/// }
///
/// assert_eq!(PayslipId::from_i64(1).unwrap().as_i64(), 1);
/// assert_eq!(PayslipId::from_i64(0), Err(platform_kernel::InvalidId));
/// ```
#[macro_export]
macro_rules! positive_id {
    ($(#[$meta:meta])* $vis:vis struct $name:ident;) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $name(i64);

        impl $name {
            pub fn from_i64(value: i64) -> ::core::result::Result<Self, $crate::InvalidId> {
                if value <= 0 {
                    return ::core::result::Result::Err($crate::InvalidId);
                }
                ::core::result::Result::Ok(Self(value))
            }

            #[must_use]
            pub fn as_i64(&self) -> i64 {
                self.0
            }
        }
    };
}

/// まだ登録していないことを表す目印。
///
/// 番号(ID)が登録したときに初めて決まるものは、登録前には番号を持たない。
/// そうしたものを `Project<Id>` のように番号の型で引数化し、登録前は番号の位置にこの目印を入れる
/// (`Project<Unsaved>`)。番号を尋ねられるのは登録済みのものだけになる。
/// どのコンテキストでも同じ意味で使うので、ここに置く
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsaved;

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
