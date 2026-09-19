use std::collections::HashMap;

use platform_auth::{current_user, require_admin};
use platform_gen::acme::timesheet::v1 as proto;
use time::Date;
use time::macros::format_description;
use timesheet_domain::project::ProjectId;
use timesheet_domain::staff::Staff;
use timesheet_domain::timesheet::{
    ReturnReason, Timesheet, TimesheetId, TimesheetStatus, WorkEntry, WorkMinutes, WorkMonth,
};
use timesheet_usecase::approval::{
    ApproveTimesheetUseCase, ListSubmittedTimesheetsUseCase, ReturnTimesheetUseCase, TimesheetView,
};
use timesheet_usecase::copies::ListProjectsUseCase;
use timesheet_usecase::my_timesheet::{
    GetMyTimesheetUseCase, MyTimesheet, SaveMyTimesheetUseCase, SubmitMyTimesheetUseCase,
};
use tonic::{Request, Response, Status};

use crate::error::{invalid_argument, to_status};

pub struct TimesheetServiceHandler {
    get_my: GetMyTimesheetUseCase,
    save_my: SaveMyTimesheetUseCase,
    submit_my: SubmitMyTimesheetUseCase,
    list_projects: ListProjectsUseCase,
    list_submitted: ListSubmittedTimesheetsUseCase,
    approve: ApproveTimesheetUseCase,
    send_back: ReturnTimesheetUseCase,
}

impl TimesheetServiceHandler {
    #[must_use]
    #[allow(clippy::too_many_arguments, reason = "サービスの RPC ごとのユースケースを受け取る")]
    pub fn new(
        get_my: GetMyTimesheetUseCase,
        save_my: SaveMyTimesheetUseCase,
        submit_my: SubmitMyTimesheetUseCase,
        list_projects: ListProjectsUseCase,
        list_submitted: ListSubmittedTimesheetsUseCase,
        approve: ApproveTimesheetUseCase,
        send_back: ReturnTimesheetUseCase,
    ) -> Self {
        Self { get_my, save_my, submit_my, list_projects, list_submitted, approve, send_back }
    }

    /// 案件番号 → 案件名。勤務表の行に案件名を添えて返すのに使う
    async fn project_names(&self) -> Result<HashMap<ProjectId, String>, Status> {
        Ok(self
            .list_projects
            .execute()
            .await
            .map_err(to_status)?
            .into_iter()
            .map(|p| (p.id(), p.name().as_str().to_owned()))
            .collect())
    }

    async fn my_response(&self, mine: MyTimesheet) -> Result<proto::Timesheet, Status> {
        let names = self.project_names().await?;
        Ok(match mine {
            MyTimesheet::NotStarted { staff, month } => proto::Timesheet {
                staff_id: staff.id().as_i64(),
                staff_name: staff.name().as_str().to_owned(),
                year: i32::from(month.year()),
                month: i32::from(month.month()),
                status: proto::TimesheetStatus::Draft.into(),
                ..proto::Timesheet::default()
            },
            MyTimesheet::Started { staff, timesheet } => to_proto(&timesheet, Some(&staff), &names),
        })
    }

    async fn view_response(&self, view: &TimesheetView) -> Result<proto::Timesheet, Status> {
        Ok(to_proto(&view.timesheet, view.staff.as_ref(), &self.project_names().await?))
    }
}

/// proto の年・月を対象月にする。as キャストは範囲外の値を黙って丸めるので、try_from で確かめる
fn month(year: i32, month: i32) -> Result<WorkMonth, Status> {
    let invalid = || Status::invalid_argument("対象月は 2000年1月〜2999年12月で指定してください");
    let year = u16::try_from(year).map_err(|_| invalid())?;
    let month = u8::try_from(month).map_err(|_| invalid())?;
    WorkMonth::new(year, month).map_err(invalid_argument)
}

fn entry(input: &proto::WorkEntryInput) -> Result<WorkEntry, Status> {
    let date =
        Date::parse(&input.date, format_description!("[year]-[month]-[day]")).map_err(|_| {
            Status::invalid_argument(format!(
                "日付は YYYY-MM-DD で指定してください: {}",
                input.date
            ))
        })?;
    Ok(WorkEntry::new(
        date,
        ProjectId::from_i64(input.project_id).map_err(invalid_argument)?,
        WorkMinutes::from_minutes(input.work_minutes).map_err(invalid_argument)?,
    ))
}

fn to_proto(
    timesheet: &Timesheet,
    staff: Option<&Staff>,
    project_names: &HashMap<ProjectId, String>,
) -> proto::Timesheet {
    let content = timesheet.content();
    let status = match timesheet.status() {
        TimesheetStatus::Draft => proto::TimesheetStatus::Draft,
        TimesheetStatus::Submitted => proto::TimesheetStatus::Submitted,
        TimesheetStatus::Approved => proto::TimesheetStatus::Approved,
    };
    let returned_reason = match timesheet {
        Timesheet::Draft(draft) => draft.returned_reason().map(|r| r.as_str().to_owned()),
        _ => None,
    };
    proto::Timesheet {
        timesheet_id: content.id().as_i64(),
        staff_id: content.staff_id().as_i64(),
        staff_name: staff.map(|s| s.name().as_str().to_owned()).unwrap_or_default(),
        year: i32::from(content.month().year()),
        month: i32::from(content.month().month()),
        status: status.into(),
        entries: content
            .entries()
            .iter()
            .map(|e| proto::WorkEntry {
                date: e.date().to_string(),
                project_id: e.project_id().as_i64(),
                project_name: project_names.get(&e.project_id()).cloned().unwrap_or_default(),
                work_minutes: e.minutes().as_minutes(),
            })
            .collect(),
        total_minutes: content.total_minutes(),
        returned_reason: returned_reason.unwrap_or_default(),
    }
}

#[tonic::async_trait]
impl proto::timesheet_service_server::TimesheetService for TimesheetServiceHandler {
    #[tracing::instrument(skip_all)]
    async fn get_my_timesheet(
        &self,
        request: Request<proto::GetMyTimesheetRequest>,
    ) -> Result<Response<proto::GetMyTimesheetResponse>, Status> {
        let user = current_user(&request)?;
        let req = request.into_inner();
        let mine =
            self.get_my.execute(&user, month(req.year, req.month)?).await.map_err(to_status)?;
        Ok(Response::new(proto::GetMyTimesheetResponse {
            timesheet: Some(self.my_response(mine).await?),
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn save_my_timesheet(
        &self,
        request: Request<proto::SaveMyTimesheetRequest>,
    ) -> Result<Response<proto::SaveMyTimesheetResponse>, Status> {
        let user = current_user(&request)?;
        let req = request.into_inner();
        let month = month(req.year, req.month)?;
        let entries = req.entries.iter().map(entry).collect::<Result<Vec<_>, _>>()?;
        let mine = self.save_my.execute(&user, month, entries).await.map_err(to_status)?;
        Ok(Response::new(proto::SaveMyTimesheetResponse {
            timesheet: Some(self.my_response(mine).await?),
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn submit_my_timesheet(
        &self,
        request: Request<proto::SubmitMyTimesheetRequest>,
    ) -> Result<Response<proto::SubmitMyTimesheetResponse>, Status> {
        let user = current_user(&request)?;
        let req = request.into_inner();
        let mine =
            self.submit_my.execute(&user, month(req.year, req.month)?).await.map_err(to_status)?;
        Ok(Response::new(proto::SubmitMyTimesheetResponse {
            timesheet: Some(self.my_response(mine).await?),
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn list_projects(
        &self,
        request: Request<proto::ListProjectsRequest>,
    ) -> Result<Response<proto::ListProjectsResponse>, Status> {
        current_user(&request)?;
        let projects = self.list_projects.execute().await.map_err(to_status)?;
        Ok(Response::new(proto::ListProjectsResponse {
            projects: projects
                .into_iter()
                .map(|p| proto::Project {
                    project_id: p.id().as_i64(),
                    name: p.name().as_str().to_owned(),
                })
                .collect(),
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn list_submitted_timesheets(
        &self,
        request: Request<proto::ListSubmittedTimesheetsRequest>,
    ) -> Result<Response<proto::ListSubmittedTimesheetsResponse>, Status> {
        require_admin(&request)?;
        let views = self.list_submitted.execute().await.map_err(to_status)?;
        let names = self.project_names().await?;
        Ok(Response::new(proto::ListSubmittedTimesheetsResponse {
            timesheets: views
                .iter()
                .map(|v| to_proto(&v.timesheet, v.staff.as_ref(), &names))
                .collect(),
            next_page_token: String::new(),
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn approve_timesheet(
        &self,
        request: Request<proto::ApproveTimesheetRequest>,
    ) -> Result<Response<proto::ApproveTimesheetResponse>, Status> {
        require_admin(&request)?;
        let id =
            TimesheetId::from_i64(request.into_inner().timesheet_id).map_err(invalid_argument)?;
        let view = self.approve.execute(id).await.map_err(to_status)?;
        Ok(Response::new(proto::ApproveTimesheetResponse {
            timesheet: Some(self.view_response(&view).await?),
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn return_timesheet(
        &self,
        request: Request<proto::ReturnTimesheetRequest>,
    ) -> Result<Response<proto::ReturnTimesheetResponse>, Status> {
        require_admin(&request)?;
        let req = request.into_inner();
        let id = TimesheetId::from_i64(req.timesheet_id).map_err(invalid_argument)?;
        let reason = ReturnReason::new(req.reason).map_err(invalid_argument)?;
        let view = self.send_back.execute(id, reason).await.map_err(to_status)?;
        Ok(Response::new(proto::ReturnTimesheetResponse {
            timesheet: Some(self.view_response(&view).await?),
        }))
    }
}
