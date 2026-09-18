//! 画面向けの読み取り(usecase の ports::queries)の MySQL 実装。
//! 集約を組み立てず、表示用の型を直接返す

use async_trait::async_trait;
use payroll_domain::project::ProjectId;
use payroll_domain::staff::StaffId;
use payroll_usecase::ports::queries::{ProjectQuery, ProjectView, StaffQuery, StaffView};
use payroll_usecase::ports::repository::RepositoryError;
use sqlx::mysql::MySqlPool;

use crate::db::{corrupted, db_err};

pub struct MySqlProjectQuery {
    pool: MySqlPool,
}

impl MySqlProjectQuery {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectQuery for MySqlProjectQuery {
    async fn list(&self) -> Result<Vec<ProjectView>, RepositoryError> {
        sqlx::query!("select id, name from projects order by id")
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?
            .into_iter()
            .map(|row| {
                Ok(ProjectView {
                    id: ProjectId::from_i64(row.id).map_err(corrupted)?,
                    name: row.name,
                })
            })
            .collect()
    }
}

pub struct MySqlStaffQuery {
    pool: MySqlPool,
}

impl MySqlStaffQuery {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StaffQuery for MySqlStaffQuery {
    async fn list(&self) -> Result<Vec<StaffView>, RepositoryError> {
        sqlx::query!("select id, email, display_name from staff order by id")
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)?
            .into_iter()
            .map(|row| {
                Ok(StaffView {
                    id: StaffId::from_i64(row.id).map_err(corrupted)?,
                    email: row.email,
                    display_name: row.display_name,
                })
            })
            .collect()
    }
}
