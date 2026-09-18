use async_trait::async_trait;
use payroll_domain::project::{NewProject, Project, ProjectId, ProjectName, ProjectRepository};
use payroll_domain::repository::RepositoryError;
use sqlx::mysql::MySqlPool;

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
    async fn insert(&self, new: &NewProject) -> Result<ProjectId, RepositoryError> {
        let result = sqlx::query!("insert into projects (name) values (?)", new.name().as_str())
            .execute(&self.pool)
            .await
            .map_err(db_err)?;

        i64::try_from(result.last_insert_id())
            .map_err(corrupted)
            .and_then(|id| ProjectId::from_i64(id).map_err(corrupted))
    }

    async fn list(&self) -> Result<Vec<Project>, RepositoryError> {
        sqlx::query!("select id, name from projects order by id")
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?
            .into_iter()
            .map(|row| {
                Ok(Project::reconstruct(
                    ProjectId::from_i64(row.id).map_err(corrupted)?,
                    ProjectName::new(row.name).map_err(corrupted)?,
                ))
            })
            .collect()
    }
}
