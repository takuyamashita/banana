//! トランザクション(usecase の ports::transaction)の MySQL 実装

use async_trait::async_trait;
use payroll_usecase::ports::repository::RepositoryError;
use payroll_usecase::ports::transaction::{Transactions, Tx};
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::db::db_err;

/// `Tx` の中身。commit せずに捨てられたら、drop 時にロールバックする
pub type MySqlTx = Transaction<'static, MySql>;

/// 受け取った `Tx` から MySQL のトランザクションを取り出す。
/// 別の実装の `Tx` が渡されるのは組み立ての誤りなので、記録はせずにエラーにする
pub(crate) fn mysql_tx(tx: &mut Tx) -> Result<&mut MySqlTx, RepositoryError> {
    tx.downcast_mut::<MySqlTx>().ok_or_else(foreign_tx)
}

fn foreign_tx() -> RepositoryError {
    RepositoryError::Unavailable("MySQL 以外のトランザクションが渡されました".to_owned())
}

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
    async fn begin(&self) -> Result<Tx, RepositoryError> {
        let tx: MySqlTx = self.pool.begin().await.map_err(db_err)?;
        Ok(Tx::new(tx))
    }

    async fn commit(&self, tx: Tx) -> Result<(), RepositoryError> {
        let tx = tx.into_inner::<MySqlTx>().map_err(|_| foreign_tx())?;
        tx.commit().await.map_err(db_err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreign_tx_is_rejected() {
        let mut tx = Tx::new("fake");
        assert!(matches!(mysql_tx(&mut tx), Err(RepositoryError::Unavailable(_))));
    }
}
