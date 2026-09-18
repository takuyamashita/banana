//! 操作を進めるために頼る先。
//!
//! - repository: 案件・派遣社員・給与明細を記録し、記録から取り出す
//! - queries: 一覧画面に出すための情報を取り出す
//! - payout_gateway: 派遣社員の口座への振込を依頼する先
//! - user_directory: 派遣社員のログイン用アカウントを発行・停止する先

pub mod payout_gateway;
pub mod queries;
pub mod repository;
pub mod user_directory;
