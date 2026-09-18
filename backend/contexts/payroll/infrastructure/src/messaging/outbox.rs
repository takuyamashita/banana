//! 出来事を outbox テーブルに記録する。記録した行は relay が SQS に送る

use async_trait::async_trait;
use payroll_domain::payslip::PayslipEvent;
use payroll_usecase::ports::events::{EventOutbox, PayrollEvent};
use payroll_usecase::ports::repository::RepositoryError;
use serde_json::Value;
use sqlx::{MySql, Transaction};

use super::payloads::PayslipFinalizedPayload;
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
fn encode(event: &PayrollEvent) -> Result<OutboxRow, serde_json::Error> {
    match event {
        PayrollEvent::Payslip {
            id,
            event: PayslipEvent::Finalized { staff_id, period, total },
        } => Ok(OutboxRow {
            aggregate_type: "payslip",
            aggregate_id: id.as_i64(),
            event_type: PayslipFinalizedPayload::EVENT_TYPE,
            payload: serde_json::to_value(PayslipFinalizedPayload {
                staff_id: staff_id.as_i64(),
                pay_year: period.year(),
                pay_month: period.month(),
                total_yen: total.as_yen(),
            })?,
        }),
    }
}

/// トランザクションの中での出来事の記録
pub struct MySqlEventOutbox<'a> {
    tx: &'a mut Transaction<'static, MySql>,
}

impl<'a> MySqlEventOutbox<'a> {
    pub(crate) fn new(tx: &'a mut Transaction<'static, MySql>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl EventOutbox for MySqlEventOutbox<'_> {
    async fn append(&mut self, event: PayrollEvent) -> Result<(), RepositoryError> {
        let row = encode(&event).map_err(corrupted)?;
        sqlx::query!(
            "insert into outbox (aggregate_type, aggregate_id, event_type, payload) values (?, ?, ?, ?)",
            row.aggregate_type,
            row.aggregate_id,
            row.event_type,
            sqlx::types::Json(&row.payload),
        )
        .execute(&mut **self.tx)
        .await
        .map_err(db_err)?;
        Ok(())
    }
}
