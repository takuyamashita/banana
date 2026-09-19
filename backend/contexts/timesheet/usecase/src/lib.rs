//! 勤怠(timesheet)で利用者が行う操作。
//!
//! - 派遣社員: 自分の月の勤務表に、日ごと・案件ごとの稼働を書き、書き終えたら申告する
//! - 管理者: 申告された勤務表を確かめ、承認するか理由を付けて差し戻す。承認した稼働は給与に知らせる
//! - 給与(payroll)で派遣社員・案件が登録されたら、その写しを記録する

pub mod approval;
pub mod copies;
pub mod error;
pub mod my_timesheet;
pub mod ports;

pub use error::UseCaseError;
