//! 一緒に確定させたい記録をまとめる単位

use async_trait::async_trait;

use super::events::EventOutbox;
use super::repository::{PayslipStore, ProjectStore, RepositoryError, StaffStore};

/// 記録をまとめる単位を始める
#[async_trait]
pub trait Transactions: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn TransactionScope>, RepositoryError>;
}

/// 記録をまとめる単位。中で行った記録は [`commit`](Self::commit) でまとめて確定する。
/// 確定せずに終わったとき(途中で失敗したときなど)は、中の記録はすべて取り消される
#[async_trait]
pub trait TransactionScope: Send {
    fn payslips(&mut self) -> Box<dyn PayslipStore + '_>;
    fn staff(&mut self) -> Box<dyn StaffStore + '_>;
    fn projects(&mut self) -> Box<dyn ProjectStore + '_>;
    fn events(&mut self) -> Box<dyn EventOutbox + '_>;
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError>;
}
