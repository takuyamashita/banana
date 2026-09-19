use async_trait::async_trait;
use payroll_domain::project::{NewProject, ProjectId};
use payroll_usecase::ports::repository::{ProjectRepository, RepositoryError};
use payroll_usecase::ports::transaction::Tx;

use crate::db::{corrupted, db_err};
use crate::transaction::mysql_tx;

pub struct MySqlProjectRepository;

#[async_trait]
impl ProjectRepository for MySqlProjectRepository {
    async fn insert(&self, tx: &mut Tx, new: &NewProject) -> Result<ProjectId, RepositoryError> {
        let tx = mysql_tx(tx)?;
        let result = sqlx::query!("insert into projects (name) values (?)", new.name().as_str())
            .execute(&mut **tx)
            .await
            .map_err(db_err)?;

        i64::try_from(result.last_insert_id())
            .map_err(corrupted)
            .and_then(|id| ProjectId::from_i64(id).map_err(corrupted))
    }
}
