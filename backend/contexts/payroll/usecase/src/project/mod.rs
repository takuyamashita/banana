use std::sync::Arc;

use payroll_domain::project::{NewProject, Project, ProjectId, ProjectName, ProjectRepository};

use crate::UseCaseError;

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
    repository: Arc<dyn ProjectRepository>,
}

impl ListProjectsUseCase {
    #[must_use]
    pub fn new(repository: Arc<dyn ProjectRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<Vec<Project>, UseCaseError> {
        Ok(self.repository.list().await?)
    }
}
