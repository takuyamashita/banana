//! 出来事を outbox テーブルに記録する。ペイロードは proto(acme.timesheet.events.v1)の JSON 形

use async_trait::async_trait;
use platform_gen::acme::timesheet::events::v1 as published;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;
use timesheet_domain::timesheet::TimesheetEvent;
use timesheet_usecase::ports::database::Db;
use timesheet_usecase::ports::events::EventOutbox;
use timesheet_usecase::ports::repository::RepositoryError;

use crate::database::mysql;
use crate::db::{corrupted, db_err};

/// 他のサービスに知らせる出来事の種類
pub const TIMESHEET_APPROVED: &str = "timesheet.approved";

/// outbox の1行になる形
struct OutboxRow {
    aggregate_type: &'static str,
    aggregate_id: i64,
    event_type: &'static str,
    payload: Value,
}

/// 出来事ごとに、outbox の行(どの集約の・何の出来事か・キューに流す JSON)を決める
fn encode(event: &TimesheetEvent) -> Result<OutboxRow, RepositoryError> {
    match event {
        TimesheetEvent::Approved { timesheet_id, staff_id, month, work, approved_at } => {
            let message = published::TimesheetApproved {
                timesheet_id: timesheet_id.as_i64(),
                staff_id: staff_id.as_i64(),
                year: i32::from(month.year()),
                month: i32::from(month.month()),
                work: work
                    .iter()
                    .map(|w| {
                        Ok(published::ProjectWork {
                            project_id: w.project_id.as_i64(),
                            work_minutes: i32::try_from(w.minutes).map_err(corrupted)?,
                        })
                    })
                    .collect::<Result<_, RepositoryError>>()?,
                approved_at: approved_at.format(&Rfc3339).map_err(corrupted)?,
            };
            Ok(OutboxRow {
                aggregate_type: "timesheet",
                aggregate_id: timesheet_id.as_i64(),
                event_type: TIMESHEET_APPROVED,
                payload: serde_json::to_value(message).map_err(corrupted)?,
            })
        }
    }
}

pub struct MySqlEventOutbox;

#[async_trait]
impl EventOutbox for MySqlEventOutbox {
    async fn append(&self, db: &mut Db, event: TimesheetEvent) -> Result<(), RepositoryError> {
        let row = encode(&event)?;
        sqlx::query!(
            "insert into outbox (aggregate_type, aggregate_id, event_type, payload, traceparent)
             values (?, ?, ?, ?, ?)",
            row.aggregate_type,
            row.aggregate_id,
            row.event_type,
            sqlx::types::Json(&row.payload),
            // 出来事を記録したリクエストのトレース。受け手(給与)の処理をこの続きにする
            platform_telemetry::current_traceparent(),
        )
        .execute(&mut *mysql(db)?)
        .await
        .map_err(db_err)?;
        Ok(())
    }
}
