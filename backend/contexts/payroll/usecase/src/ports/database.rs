//! 記録の書き込み先(トランザクションと接続)

use std::any::Any;
use std::ops::{Deref, DerefMut};

use async_trait::async_trait;

use super::repository::RepositoryError;

/// 書き込み先を用意する。
///
/// 一緒に確定させたい記録は、同じ [`transaction`](Self::transaction) に書いて最後に
/// [`Transaction::commit`] する。1件だけ書くときは [`connection`](Self::connection) に書けば、
/// 書いた時点で確定する
#[async_trait]
pub trait Database: Send + Sync {
    async fn transaction(&self) -> Result<Transaction, RepositoryError>;
    async fn connection(&self) -> Result<Db, RepositoryError>;
}

/// 記録の書き込み先。トランザクションか接続かは、書き込む側からは区別しない
pub struct Db(Box<dyn DbHandle>);

impl Db {
    /// 書き込み先の実装が、自分の接続やトランザクションを包む
    pub fn new(handle: impl DbHandle) -> Self {
        Self(Box::new(handle))
    }

    /// 書き込み先の実装が、包んだものを取り出す。別の実装のものなら `None`
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.0.as_any_mut().downcast_mut()
    }
}

/// 始めたトランザクション。書き込み先([`Db`])として渡せる。
///
/// 書いた記録は [`commit`](Self::commit) でまとめて確定する。確定せずに終わったとき
/// (途中で失敗したときなど)は、書いた記録はすべて取り消される
pub struct Transaction(Db);

impl Transaction {
    pub fn new(handle: impl DbHandle) -> Self {
        Self(Db::new(handle))
    }

    pub async fn commit(self) -> Result<(), RepositoryError> {
        self.0.0.commit().await
    }
}

impl Deref for Transaction {
    type Target = Db;

    fn deref(&self) -> &Db {
        &self.0
    }
}

impl DerefMut for Transaction {
    fn deref_mut(&mut self) -> &mut Db {
        &mut self.0
    }
}

/// 書き込み先の実装が包むもの
#[async_trait]
pub trait DbHandle: Send + 'static {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    /// トランザクションなら確定する
    async fn commit(self: Box<Self>) -> Result<(), RepositoryError>;
}
