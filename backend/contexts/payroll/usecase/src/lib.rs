//! 給与計算(payroll)で利用者が行う操作。
//!
//! - 管理者: 案件と派遣社員を登録し、派遣社員ごとに毎月の給与を確定する
//! - 派遣社員: 自分の給与明細を見る(他人の給与明細は見られない)
//! - 給与の確定を受けて、派遣社員の口座へ支給額を振り込む

pub mod error;
pub mod payslip;
pub mod ports;
pub mod project;
pub mod staff;

pub use error::UseCaseError;
