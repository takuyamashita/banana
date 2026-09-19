//! トランザクション(usecase の ports::transaction)の MySQL 実装

use async_trait::async_trait;
use payroll_usecase::ports::repository::RepositoryError;
use payroll_usecase::ports::transaction::Transactions;
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::db::db_err;

/// リポジトリと outbox が受け取るトランザクション。
/// commit せずに捨てられたら、drop 時にロールバックする
pub type MySqlTx = Transaction<'static, MySql>;

pub struct MySqlTransactions {
    pool: MySqlPool,
}

impl MySqlTransactions {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Transactions for MySqlTransactions {
    type Tx = MySqlTx;

    async fn begin(&self) -> Result<MySqlTx, RepositoryError> {
        self.pool.begin().await.map_err(db_err)
    }

    async fn commit(&self, tx: MySqlTx) -> Result<(), RepositoryError> {
        tx.commit().await.map_err(db_err)
    }
}
