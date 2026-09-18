use std::sync::Arc;

use payroll_domain::project::{NewProject, ProjectId, ProjectName};

use crate::UseCaseError;
use crate::ports::queries::{ProjectQuery, ProjectView};
use crate::ports::repository::ProjectRepository;

pub struct CreateProjectUseCase {
    repository: Arc<dyn ProjectRepository>,
}

impl CreateProjectUseCase {
    #[must_use]
    pub fn new(repository: Arc<dyn ProjectRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, name: ProjectName) -> Result<ProjectId, UseCaseError> {
        Ok(self.repository.insert(&NewProject::new(name)).await?)
    }
}

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
