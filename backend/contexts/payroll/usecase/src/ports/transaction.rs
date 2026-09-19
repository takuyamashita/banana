//! 一緒に確定させたい記録をまとめる単位(トランザクション)

use async_trait::async_trait;

use super::repository::RepositoryError;

/// トランザクションを始め、確定させる。
///
/// 始めたトランザクション(`Tx`)を記録(`insert`・`update`・`append`)に渡すと、その記録は
/// [`commit`](Self::commit) でまとめて確定する。確定せずに終わったトランザクション
/// (途中で失敗したときなど)の記録は、すべて取り消される
#[async_trait]
pub trait Transactions: Send + Sync + 'static {
    /// 始めたトランザクション
    type Tx: Send + 'static;

    async fn begin(&self) -> Result<Self::Tx, RepositoryError>;
    async fn commit(&self, tx: Self::Tx) -> Result<(), RepositoryError>;
}
