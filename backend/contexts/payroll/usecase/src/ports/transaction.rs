//! 一緒に確定させたい記録をまとめる単位(トランザクション)

use std::any::Any;

use async_trait::async_trait;

use super::repository::RepositoryError;

/// 始めたトランザクション。
///
/// 記録(`insert`・`update`・`append`)に渡すと、その記録は [`Transactions::commit`] で
/// まとめて確定する。確定せずに終わったトランザクション(途中で失敗したときなど)の記録は、
/// すべて取り消される
pub struct Tx(Box<dyn Any + Send>);

impl Tx {
    /// 記録先の実装が、自分のトランザクションを包む
    pub fn new<T: Any + Send>(inner: T) -> Self {
        Self(Box::new(inner))
    }

    /// 記録先の実装が、包んだトランザクションを取り出す。別の実装のものなら `None`
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.0.downcast_mut()
    }

    /// 記録先の実装が、確定のために包みを解く。別の実装のものなら `Err` で元の `Tx` を返す
    pub fn into_inner<T: Any>(self) -> Result<T, Self> {
        self.0.downcast().map(|inner| *inner).map_err(Self)
    }
}

/// トランザクションを始め、確定させる
#[async_trait]
pub trait Transactions: Send + Sync {
    async fn begin(&self) -> Result<Tx, RepositoryError>;
    async fn commit(&self, tx: Tx) -> Result<(), RepositoryError>;
}
