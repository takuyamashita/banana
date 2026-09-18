use std::sync::Arc;

use payroll_domain::project::{NewProject, ProjectId, ProjectName};

use crate::UseCaseError;
use crate::ports::queries::{ProjectQuery, ProjectView};
use crate::ports::repository::ProjectRepository;

/// 管理者が案件を登録する
pub struct CreateProjectUseCase {
    repository: Arc<dyn ProjectRepository>,
}

impl CreateProjectUseCase {
    #[must_use]
    pub fn new(repository: Arc<dyn ProjectRepository>) -> Self {
        Self { repository }
    }

    /// 案件を登録し、振られた案件番号を返す
    pub async fn execute(&self, name: ProjectName) -> Result<ProjectId, UseCaseError> {
        Ok(self.repository.insert(&NewProject::new(name)).await?)
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
