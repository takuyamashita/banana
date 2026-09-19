use async_trait::async_trait;
use payroll_domain::project::{NewProject, ProjectId};
use payroll_usecase::ports::database::Db;
use payroll_usecase::ports::repository::{ProjectRepository, RepositoryError};

use crate::database::mysql;
use crate::db::{corrupted, db_err};

pub struct MySqlProjectRepository;

#[async_trait]
impl ProjectRepository for MySqlProjectRepository {
    async fn insert(&self, db: &mut Db, new: &NewProject) -> Result<ProjectId, RepositoryError> {
        let conn = mysql(db)?;
        let result = sqlx::query!("insert into projects (name) values (?)", new.name().as_str())
            .execute(&mut *conn)
            .await
            .map_err(db_err)?;

        i64::try_from(result.last_insert_id())
            .map_err(corrupted)
            .and_then(|id| ProjectId::from_i64(id).map_err(corrupted))
    }
}
