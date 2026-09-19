use payroll_usecase::user::{CreateAdminUserInput, CreateAdminUserUseCase};
use platform_gen::acme::payroll::v1 as proto;
use platform_kernel::Email;
use tonic::{Request, Response, Status};

use crate::auth::require_admin;
use crate::error::{invalid_argument, to_status};

pub struct UserServiceHandler {
    create_admin_user: CreateAdminUserUseCase,
}

impl UserServiceHandler {
    #[must_use]
    pub fn new(create_admin_user: CreateAdminUserUseCase) -> Self {
        Self { create_admin_user }
    }
}

#[tonic::async_trait]
impl proto::user_service_server::UserService for UserServiceHandler {
    #[tracing::instrument(skip_all)]
    async fn create_admin_user(
        &self,
        request: Request<proto::CreateAdminUserRequest>,
    ) -> Result<Response<proto::CreateAdminUserResponse>, Status> {
        require_admin(&request)?;
        let req = request.into_inner();

        let input = CreateAdminUserInput {
            email: Email::parse(req.email).map_err(invalid_argument)?,
            temporary_password: req.temporary_password,
        };
        let user_id = self.create_admin_user.execute(input).await.map_err(to_status)?;

        Ok(Response::new(proto::CreateAdminUserResponse { user_id: user_id.as_str().to_owned() }))
    }
}
