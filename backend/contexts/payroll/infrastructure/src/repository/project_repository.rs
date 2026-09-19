use async_trait::async_trait;
use payroll_domain::project::{NewProject, Project, ProjectId, ProjectName};
use payroll_usecase::ports::database::Db;
use payroll_usecase::ports::repository::{ProjectRepository, RepositoryError};
use sqlx::mysql::MySqlPool;

use crate::database::mysql;
use crate::db::{corrupted, db_err};

pub struct MySqlProjectRepository {
    pool: MySqlPool,
}

impl MySqlProjectRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for MySqlProjectRepository {
    async fn find(&self, id: ProjectId) -> Result<Option<Project>, RepositoryError> {
        let Some(row) = sqlx::query!("select id, name from projects where id = ?", id.as_i64())
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?
        else {
            return Ok(None);
        };
        Ok(Some(Project::reconstruct(
            ProjectId::from_i64(row.id).map_err(corrupted)?,
            ProjectName::new(row.name).map_err(corrupted)?,
        )))
    }

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
