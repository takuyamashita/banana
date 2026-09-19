//! 案件の登録と一覧

use std::sync::Arc;

use payroll_domain::project::{NewProject, ProjectId, ProjectName};

use crate::UseCaseError;
use crate::ports::database::Database;
use crate::ports::events::{EventOutbox, PayrollEvent};
use crate::ports::queries::{ProjectQuery, ProjectView};
use crate::ports::repository::ProjectRepository;

/// 管理者が案件を登録する。登録したことは他の業務(勤怠など)に知らせる
pub struct CreateProjectUseCase {
    repository: Arc<dyn ProjectRepository>,
    outbox: Arc<dyn EventOutbox>,
    db: Arc<dyn Database>,
}

impl CreateProjectUseCase {
    #[must_use]
    pub fn new(
        repository: Arc<dyn ProjectRepository>,
        outbox: Arc<dyn EventOutbox>,
        db: Arc<dyn Database>,
    ) -> Self {
        Self { repository, outbox, db }
    }

    /// 案件を登録し、振られた案件番号を返す。案件と、登録したという出来事は一緒に記録する
    pub async fn execute(&self, name: ProjectName) -> Result<ProjectId, UseCaseError> {
        let new = NewProject::new(name);
        let mut tx = self.db.transaction().await?;
        let id = self.repository.insert(&mut tx, &new).await?;
        self.outbox.append(&mut tx, PayrollEvent::Project(new.created_as(id))).await?;
        tx.commit().await?;
        Ok(id)
    }
}

/// 管理者が、登録済みの案件を一覧する
pub struct ListProjectsUseCase {
    query: Arc<dyn ProjectQuery>,
}

impl ListProjectsUseCase {
    #[must_use]
    pub fn new(query: Arc<dyn ProjectQuery>) -> Self {
        Self { query }
    }

    pub async fn execute(&self) -> Result<Vec<ProjectView>, UseCaseError> {
        Ok(self.query.list().await?)
    }
}
