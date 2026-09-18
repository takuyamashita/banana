//! payroll の infrastructure。usecase の ports(リポジトリ・クエリ・外部API)を実装する

pub mod db;
pub mod external;
pub mod messaging;
pub mod query;
pub mod repository;

pub use db::{MIGRATOR, connect};
