//! 給与(payroll)から届いた出来事を読み、派遣社員・案件の写しを記録する

use async_trait::async_trait;
use platform_gen::acme::payroll::events::v1 as payroll;
use platform_kernel::UserId;
use platform_messaging::envelope::OutboxEnvelope;
use timesheet_domain::project::{Project, ProjectId, ProjectName};
use timesheet_domain::staff::{Staff, StaffId, StaffName};
use timesheet_usecase::copies::{RecordProjectUseCase, RecordStaffUseCase};

/// 給与の出来事の種類(proto の acme.payroll.events.v1 に書いてあるもの)
const STAFF_REGISTERED: &str = "staff.registered";
const PROJECT_CREATED: &str = "project.created";

/// 給与から届いた出来事の受け手。知らない種類は読み飛ばす(給与が出来事を増やしても止まらない)。
/// 読めない出来事は失敗にして、キューの再配信と DLQ に任せる
pub struct PayrollEventHandler {
    record_staff: RecordStaffUseCase,
    record_project: RecordProjectUseCase,
}

impl PayrollEventHandler {
    #[must_use]
    pub fn new(record_staff: RecordStaffUseCase, record_project: RecordProjectUseCase) -> Self {
        Self { record_staff, record_project }
    }
}

#[async_trait]
impl platform_messaging::consumer::Handler for PayrollEventHandler {
    async fn handle(&self, body: &str) -> anyhow::Result<()> {
        let envelope: OutboxEnvelope<serde_json::Value> = serde_json::from_str(body)?;
        let event_id = envelope.event_id;
        match envelope.event_type.as_str() {
            STAFF_REGISTERED => {
                let event: payroll::StaffRegistered = serde_json::from_value(envelope.payload)?;
                let staff = Staff::new(
                    StaffId::from_i64(event.staff_id)?,
                    UserId::parse(event.user_id)?,
                    StaffName::new(event.display_name)?,
                );
                self.record_staff.execute(staff).await?;
                tracing::info!(event_id, staff_id = event.staff_id, "staff recorded");
            }
            PROJECT_CREATED => {
                let event: payroll::ProjectCreated = serde_json::from_value(envelope.payload)?;
                let project = Project::new(
                    ProjectId::from_i64(event.project_id)?,
                    ProjectName::new(event.name)?,
                );
                self.record_project.execute(project).await?;
                tracing::info!(event_id, project_id = event.project_id, "project recorded");
            }
            other => tracing::info!(event_id, event_type = other, "not for this consumer, skipped"),
        }
        Ok(())
    }
}
