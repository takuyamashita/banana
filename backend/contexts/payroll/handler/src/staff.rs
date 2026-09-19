use payroll_domain::staff::{DisplayName, Staff};
use payroll_usecase::ports::queries::StaffView;
use payroll_usecase::staff::{
    CreateStaffInput, CreateStaffUseCase, GetMeUseCase, ListStaffUseCase,
};
use platform_gen::acme::payroll::v1 as proto;
use platform_kernel::Email;
use tonic::{Request, Response, Status};

use crate::auth::{current_user, require_admin};
use crate::error::{invalid_argument, to_status};

pub struct StaffServiceHandler {
    create_staff: CreateStaffUseCase,
    list_staff: ListStaffUseCase,
    get_me: GetMeUseCase,
}

impl StaffServiceHandler {
    #[must_use]
    pub fn new(
        create_staff: CreateStaffUseCase,
        list_staff: ListStaffUseCase,
        get_me: GetMeUseCase,
    ) -> Self {
        Self { create_staff, list_staff, get_me }
    }
}

#[tonic::async_trait]
impl proto::staff_service_server::StaffService for StaffServiceHandler {
    #[tracing::instrument(skip_all)]
    async fn create_staff(
        &self,
        request: Request<proto::CreateStaffRequest>,
    ) -> Result<Response<proto::CreateStaffResponse>, Status> {
        require_admin(&request)?;
        let req = request.into_inner();

        let input = CreateStaffInput {
            email: Email::parse(req.email).map_err(invalid_argument)?,
            display_name: DisplayName::new(req.display_name).map_err(invalid_argument)?,
            temporary_password: req.temporary_password,
        };
        let staff_id = self.create_staff.execute(input).await.map_err(to_status)?;

        Ok(Response::new(proto::CreateStaffResponse { staff_id: staff_id.as_i64() }))
    }

    #[tracing::instrument(skip_all)]
    async fn list_staff(
        &self,
        request: Request<proto::ListStaffRequest>,
    ) -> Result<Response<proto::ListStaffResponse>, Status> {
        require_admin(&request)?;
        let staff = self.list_staff.execute().await.map_err(to_status)?;
        Ok(Response::new(proto::ListStaffResponse {
            staff: staff.iter().map(view_to_proto).collect(),
            // 今は全件を返すので、続きはない
            next_page_token: String::new(),
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn get_me(
        &self,
        request: Request<proto::GetMeRequest>,
    ) -> Result<Response<proto::GetMeResponse>, Status> {
        let user = current_user(&request)?;
        let staff = self.get_me.execute(&user).await.map_err(to_status)?;
        Ok(Response::new(proto::GetMeResponse {
            staff: staff.as_ref().map(to_proto),
            roles: user.roles.iter().map(|r| r.as_str().to_owned()).collect(),
        }))
    }
}

fn to_proto(s: &Staff) -> proto::Staff {
    proto::Staff {
        staff_id: s.id().as_i64(),
        email: s.email().as_str().to_owned(),
        display_name: s.display_name().as_str().to_owned(),
    }
}

fn view_to_proto(s: &StaffView) -> proto::Staff {
    proto::Staff {
        staff_id: s.id.as_i64(),
        email: s.email.clone(),
        display_name: s.display_name.clone(),
    }
}
