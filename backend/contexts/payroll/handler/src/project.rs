use payroll_domain::project::ProjectName;
use payroll_usecase::ports::transaction::Transactions;
use payroll_usecase::project::{CreateProjectUseCase, ListProjectsUseCase};
use platform_gen::acme::payroll::v1 as proto;
use tonic::{Request, Response, Status};

use crate::auth::require_admin;
use crate::error::{invalid_argument, to_status};

pub struct ProjectServiceHandler<T: Transactions> {
    create_project: CreateProjectUseCase<T>,
    list_projects: ListProjectsUseCase,
}

impl<T: Transactions> ProjectServiceHandler<T> {
    #[must_use]
    pub fn new(
        create_project: CreateProjectUseCase<T>,
        list_projects: ListProjectsUseCase,
    ) -> Self {
        Self { create_project, list_projects }
    }
}

#[tonic::async_trait]
impl<T: Transactions> proto::project_service_server::ProjectService for ProjectServiceHandler<T> {
    async fn create_project(
        &self,
        request: Request<proto::CreateProjectRequest>,
    ) -> Result<Response<proto::CreateProjectResponse>, Status> {
        require_admin(&request)?;
        let name = ProjectName::new(request.into_inner().name).map_err(invalid_argument)?;
        let id = self.create_project.execute(name).await.map_err(to_status)?;
        Ok(Response::new(proto::CreateProjectResponse { project_id: id.as_i64() }))
    }

    async fn list_projects(
        &self,
        request: Request<proto::ListProjectsRequest>,
    ) -> Result<Response<proto::ListProjectsResponse>, Status> {
        require_admin(&request)?;
        let projects = self.list_projects.execute().await.map_err(to_status)?;
        Ok(Response::new(proto::ListProjectsResponse {
            projects: projects
                .iter()
                .map(|p| proto::Project { project_id: p.id.as_i64(), name: p.name.clone() })
                .collect(),
        }))
    }
}
