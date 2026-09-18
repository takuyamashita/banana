use async_trait::async_trait;
use payroll_domain::project::{NewProject, ProjectId};
use payroll_usecase::ports::repository::{ProjectRepository, RepositoryError};
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
}
