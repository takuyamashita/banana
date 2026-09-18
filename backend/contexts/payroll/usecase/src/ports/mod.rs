//! usecase が外部に求めるもの(ports)。実装は infrastructure に置く。
//!
//! - repository: 集約の出し入れ(書き込み側)。集約を丸ごと保存・復元する
//! - queries: 画面向けの読み取り。集約を組み立てず、表示用の型を直接返す
//! - payout_gateway・user_directory: 外部 API

pub mod payout_gateway;
pub mod queries;
pub mod repository;
pub mod user_directory;
