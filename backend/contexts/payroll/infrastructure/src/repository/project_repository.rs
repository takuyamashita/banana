use async_trait::async_trait;
use payroll_domain::project::{NewProject, ProjectId};
use payroll_usecase::ports::repository::{ProjectRepository, RepositoryError};

use crate::db::{corrupted, db_err};
use crate::transaction::MySqlTx;

pub struct MySqlProjectRepository;

#[async_trait]
impl ProjectRepository<MySqlTx> for MySqlProjectRepository {
    async fn insert(
        &self,
        tx: &mut MySqlTx,
        new: &NewProject,
    ) -> Result<ProjectId, RepositoryError> {
        let result = sqlx::query!("insert into projects (name) values (?)", new.name().as_str())
            .execute(&mut **tx)
            .await
            .map_err(db_err)?;

        i64::try_from(result.last_insert_id())
            .map_err(corrupted)
            .and_then(|id| ProjectId::from_i64(id).map_err(corrupted))
    }
}
