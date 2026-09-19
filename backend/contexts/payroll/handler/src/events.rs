//! 勤怠(timesheet)から届いた出来事を読み、承認された稼働を記録する

use async_trait::async_trait;
use payroll_domain::payslip::{PayPeriod, WorkMinutes};
use payroll_domain::project::ProjectId;
use payroll_domain::staff::StaffId;
use payroll_domain::work::{ApprovedWork, ProjectWork};
use payroll_usecase::work::RecordApprovedWorkUseCase;
use platform_gen::acme::timesheet::events::v1 as timesheet;
use platform_messaging::envelope::OutboxEnvelope;

/// 勤怠の出来事の種類(proto の acme.timesheet.events.v1 に書いてあるもの)
const TIMESHEET_APPROVED: &str = "timesheet.approved";

/// 勤怠から届いた出来事の受け手。知らない種類は読み飛ばす(勤怠が出来事を増やしても止まらない)。
/// 読めない出来事は失敗にして、キューの再配信と DLQ に任せる
pub struct TimesheetEventHandler {
    record: RecordApprovedWorkUseCase,
}

impl TimesheetEventHandler {
    #[must_use]
    pub fn new(record: RecordApprovedWorkUseCase) -> Self {
        Self { record }
    }
}

#[async_trait]
impl platform_messaging::consumer::Handler for TimesheetEventHandler {
    async fn handle(&self, body: &str) -> anyhow::Result<()> {
        let envelope: OutboxEnvelope<serde_json::Value> = serde_json::from_str(body)?;
        let event_id = envelope.event_id;
        if envelope.event_type != TIMESHEET_APPROVED {
            tracing::info!(
                event_id,
                event_type = envelope.event_type,
                "not for this consumer, skipped"
            );
            return Ok(());
        }
        let event: timesheet::TimesheetApproved = serde_json::from_value(envelope.payload)?;
        let projects = event
            .work
            .iter()
            .map(|w| {
                Ok(ProjectWork {
                    project_id: ProjectId::from_i64(w.project_id)?,
                    minutes: WorkMinutes::from_minutes(u32::try_from(w.work_minutes)?)?,
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let work = ApprovedWork::new(
            StaffId::from_i64(event.staff_id)?,
            PayPeriod::new(u16::try_from(event.year)?, u8::try_from(event.month)?)?,
            projects,
        )?;
        self.record.execute(work).await?;
        tracing::info!(event_id, timesheet_id = event.timesheet_id, "approved work recorded");
        Ok(())
    }
}
