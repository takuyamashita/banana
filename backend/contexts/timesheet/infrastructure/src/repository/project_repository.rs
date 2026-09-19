use async_trait::async_trait;
use sqlx::mysql::MySqlPool;
use timesheet_domain::project::{Project, ProjectId, ProjectName};
use timesheet_usecase::ports::database::Db;
use timesheet_usecase::ports::repository::{ProjectRepository, RepositoryError};

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

struct Row {
    id: i64,
    name: String,
}

fn to_project(row: Row) -> Result<Project, RepositoryError> {
    Ok(Project::new(
        ProjectId::from_i64(row.id).map_err(corrupted)?,
        ProjectName::new(row.name).map_err(corrupted)?,
    ))
}

#[async_trait]
impl ProjectRepository for MySqlProjectRepository {
    async fn find(&self, id: ProjectId) -> Result<Option<Project>, RepositoryError> {
        sqlx::query_as!(Row, "select id, name from projects where id = ?", id.as_i64())
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?
            .map(to_project)
            .transpose()
    }

    async fn list(&self) -> Result<Vec<Project>, RepositoryError> {
        sqlx::query_as!(Row, "select id, name from projects order by id")
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?
            .into_iter()
            .map(to_project)
            .collect()
    }

    async fn save(&self, db: &mut Db, project: &Project) -> Result<(), RepositoryError> {
        sqlx::query!(
            "insert into projects (id, name) values (?, ?) as new
             on duplicate key update name = new.name",
            project.id().as_i64(),
            project.name().as_str(),
        )
        .execute(&mut *mysql(db)?)
        .await
        .map_err(db_err)?;
        Ok(())
    }
}
