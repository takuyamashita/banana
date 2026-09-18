//! payroll のユースケース。
//!
//! ロール判定(管理者か)は handler の仕事、明細が本人のものかといったリソース単位の認可は
//! リポジトリを引く必要があるので usecase で行う。

pub mod error;
pub mod payslip;
pub mod ports;
pub mod project;
pub mod staff;

pub use error::UseCaseError;
