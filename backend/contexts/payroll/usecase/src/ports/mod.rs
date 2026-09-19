//! 操作を進めるために頼る先。
//!
//! - repository: 案件・派遣社員・給与明細を記録し、記録から取り出す
//! - database: 記録の書き込み先(トランザクションと接続)
//! - events: 他の業務が知るべき出来事の記録
//! - queries: 一覧画面に出すための情報を取り出す
//! - payout_gateway: 派遣社員の口座への振込を依頼する先
//! - user_directory: 派遣社員のログイン用アカウントを発行・停止する先

pub mod database;
pub mod events;
pub mod payout_gateway;
pub mod queries;
pub mod repository;
pub mod user_directory;
