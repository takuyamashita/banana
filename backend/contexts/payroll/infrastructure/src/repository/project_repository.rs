use async_trait::async_trait;
use payroll_domain::project::{NewProject, ProjectId};
use payroll_usecase::ports::repository::{ProjectStore, RepositoryError};
use sqlx::{MySql, Transaction};

use crate::db::{corrupted, db_err};

/// トランザクションの中での案件の記録
pub struct MySqlProjectStore<'a> {
    tx: &'a mut Transaction<'static, MySql>,
}

impl<'a> MySqlProjectStore<'a> {
    pub(crate) fn new(tx: &'a mut Transaction<'static, MySql>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl ProjectStore for MySqlProjectStore<'_> {
    async fn insert(&mut self, new: &NewProject) -> Result<ProjectId, RepositoryError> {
        let result = sqlx::query!("insert into projects (name) values (?)", new.name().as_str())
            .execute(&mut **self.tx)
            .await
            .map_err(db_err)?;

        i64::try_from(result.last_insert_id())
            .map_err(corrupted)
            .and_then(|id| ProjectId::from_i64(id).map_err(corrupted))
    }
}
