//! 書き込み先(usecase の ports::database)の MySQL 実装

use std::any::Any;

use async_trait::async_trait;
use payroll_usecase::ports::database::{Database, Db, DbHandle, Transaction};
use payroll_usecase::ports::repository::RepositoryError;
use sqlx::MySql;
use sqlx::mysql::{MySqlConnection, MySqlPool};
use sqlx::pool::PoolConnection;

use crate::db::db_err;

/// `Db` の中身。トランザクションは commit せずに捨てられたら、drop 時にロールバックする
enum MySqlDb {
    Transaction(sqlx::Transaction<'static, MySql>),
    Connection(PoolConnection<MySql>),
}

#[async_trait]
impl DbHandle for MySqlDb {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    async fn commit(self: Box<Self>) -> Result<(), RepositoryError> {
        match *self {
            Self::Transaction(tx) => tx.commit().await.map_err(db_err),
            // 接続に書いた記録は、書いた時点で確定している
            Self::Connection(_) => Ok(()),
        }
    }
}

/// 受け取った `Db` から MySQL の接続を取り出す。トランザクションなら、その中の接続になる。
/// 別の実装の `Db` が渡されるのは組み立ての誤りなので、記録はせずに Internal にする
/// (Unavailable にすると、直らないのに再試行される)
pub(crate) fn mysql(db: &mut Db) -> Result<&mut MySqlConnection, RepositoryError> {
    match db.downcast_mut::<MySqlDb>() {
        Some(MySqlDb::Transaction(tx)) => Ok(&mut **tx),
        Some(MySqlDb::Connection(conn)) => Ok(&mut **conn),
        None => Err(foreign_db()),
    }
}

/// トランザクションの中の接続を取り出す。ロックして読む(select ... for update)ときに使う。
/// 接続が渡されたら、ロックは文が終わった時点で外れて意味がないので、Internal にする
/// (usecase がトランザクションを張り忘れたことを、テストで検出できるようにする)
pub(crate) fn mysql_tx(db: &mut Db) -> Result<&mut MySqlConnection, RepositoryError> {
    match db.downcast_mut::<MySqlDb>() {
        Some(MySqlDb::Transaction(tx)) => Ok(&mut **tx),
        Some(MySqlDb::Connection(_)) => Err(RepositoryError::Internal(
            "ロックして読むにはトランザクションが必要です".to_owned(),
        )),
        None => Err(foreign_db()),
    }
}

fn foreign_db() -> RepositoryError {
    RepositoryError::Internal("MySQL 以外の書き込み先が渡されました".to_owned())
}

pub struct MySqlDatabase {
    pool: MySqlPool,
}

impl MySqlDatabase {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Database for MySqlDatabase {
    async fn transaction(&self) -> Result<Transaction, RepositoryError> {
        let tx = self.pool.begin().await.map_err(db_err)?;
        Ok(Transaction::new(MySqlDb::Transaction(tx)))
    }

    async fn connection(&self) -> Result<Db, RepositoryError> {
        let conn = self.pool.acquire().await.map_err(db_err)?;
        Ok(Db::new(MySqlDb::Connection(conn)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Foreign;

    #[async_trait]
    impl DbHandle for Foreign {
        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        async fn commit(self: Box<Self>) -> Result<(), RepositoryError> {
            Ok(())
        }
    }

    #[test]
    fn foreign_db_is_rejected() {
        let mut db = Db::new(Foreign);
        assert!(matches!(mysql(&mut db), Err(RepositoryError::Internal(_))));
        assert!(matches!(mysql_tx(&mut db), Err(RepositoryError::Internal(_))));
    }
}
