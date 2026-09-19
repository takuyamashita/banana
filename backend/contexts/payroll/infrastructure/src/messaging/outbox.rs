//! 出来事を outbox テーブルに記録する。記録した行は relay が SQS に送る

use async_trait::async_trait;
use payroll_domain::payslip::PayslipEvent;
use payroll_usecase::ports::database::Db;
use payroll_usecase::ports::events::{EventOutbox, PayrollEvent};
use payroll_usecase::ports::repository::RepositoryError;
use serde_json::Value;
use time::format_description::well_known::Rfc3339;

use super::payloads::PayslipFinalizedPayload;
use crate::database::mysql;
use crate::db::{corrupted, db_err};

/// outbox の1行になる形
struct OutboxRow {
    aggregate_type: &'static str,
    aggregate_id: i64,
    event_type: &'static str,
    payload: Value,
}

/// 出来事ごとに、outbox の行(どの集約の・何の出来事か・キューに流す JSON)を決める。
/// 出来事の種類が増えたらここに足す
fn encode(event: &PayrollEvent) -> Result<OutboxRow, RepositoryError> {
    match event {
        PayrollEvent::Payslip(PayslipEvent::Finalized {
            payslip_id,
            staff_id,
            period,
            total,
            finalized_at,
        }) => Ok(OutboxRow {
            aggregate_type: "payslip",
            aggregate_id: payslip_id.as_i64(),
            event_type: PayslipFinalizedPayload::EVENT_TYPE,
            payload: serde_json::to_value(PayslipFinalizedPayload {
                payslip_id: payslip_id.as_i64(),
                staff_id: staff_id.as_i64(),
                pay_year: period.year(),
                pay_month: period.month(),
                total_yen: total.as_yen(),
                finalized_at: finalized_at.format(&Rfc3339).map_err(corrupted)?,
            })
            .map_err(corrupted)?,
        }),
    }
}

pub struct MySqlEventOutbox;

#[async_trait]
impl EventOutbox for MySqlEventOutbox {
    async fn append(&self, db: &mut Db, event: PayrollEvent) -> Result<(), RepositoryError> {
        let conn = mysql(db)?;
        let row = encode(&event)?;
        sqlx::query!(
            "insert into outbox (aggregate_type, aggregate_id, event_type, payload, traceparent)
             values (?, ?, ?, ?, ?)",
            row.aggregate_type,
            row.aggregate_id,
            row.event_type,
            sqlx::types::Json(&row.payload),
            // 出来事を記録したリクエストのトレース。受け手(振込など)のトレースをこの続きにする
            platform_telemetry::current_traceparent(),
        )
        .execute(&mut *conn)
        .await
        .map_err(db_err)?;
        Ok(())
    }
}
