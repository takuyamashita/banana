//! 記録をまとめる単位(usecase の ports::transaction)の MySQL 実装。
//! 1つの sqlx のトランザクションを、各記録先と outbox で共有する

use async_trait::async_trait;
use payroll_usecase::ports::events::EventOutbox;
use payroll_usecase::ports::repository::{PayslipStore, ProjectStore, RepositoryError, StaffStore};
use payroll_usecase::ports::transaction::{TransactionScope, Transactions};
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::db::db_err;
use crate::messaging::outbox::MySqlEventOutbox;
use crate::repository::{MySqlPayslipStore, MySqlProjectStore, MySqlStaffStore};

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
    async fn begin(&self) -> Result<Box<dyn TransactionScope>, RepositoryError> {
        let tx = self.pool.begin().await.map_err(db_err)?;
        Ok(Box::new(MySqlTransactionScope { tx }))
    }
}

/// commit せずに捨てられたら、sqlx の Transaction が drop 時にロールバックする
struct MySqlTransactionScope {
    tx: Transaction<'static, MySql>,
}

#[async_trait]
impl TransactionScope for MySqlTransactionScope {
    fn payslips(&mut self) -> Box<dyn PayslipStore + '_> {
        Box::new(MySqlPayslipStore::new(&mut self.tx))
    }

    fn staff(&mut self) -> Box<dyn StaffStore + '_> {
        Box::new(MySqlStaffStore::new(&mut self.tx))
    }

    fn projects(&mut self) -> Box<dyn ProjectStore + '_> {
        Box::new(MySqlProjectStore::new(&mut self.tx))
    }

    fn events(&mut self) -> Box<dyn EventOutbox + '_> {
        Box::new(MySqlEventOutbox::new(&mut self.tx))
    }

    async fn commit(self: Box<Self>) -> Result<(), RepositoryError> {
        self.tx.commit().await.map_err(db_err)
    }
}
