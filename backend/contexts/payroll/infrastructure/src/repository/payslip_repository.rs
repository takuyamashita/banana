use async_trait::async_trait;
use payroll_domain::payslip::{
    NewPayslip, PayPeriod, Payslip, PayslipEvent, PayslipId, PayslipLine, PayslipStatus,
    WorkMinutes,
};
use payroll_domain::project::ProjectId;
use payroll_domain::staff::StaffId;
use payroll_usecase::ports::repository::{PayslipRepository, RepositoryError};
use platform_kernel::Money;
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::db::{corrupted, db_err};
use crate::messaging::payloads::PayslipFinalizedPayload;

pub struct MySqlPayslipRepository {
    pool: MySqlPool,
}

impl MySqlPayslipRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

/// 明細と明細行を JOIN した1行。集約内の JOIN なので許可されている
struct JoinedRow {
    id: i64,
    staff_id: i64,
    pay_year: u16,
    pay_month: u8,
    status: String,
    project_id: i64,
    work_minutes: u32,
    hourly_rate: i64,
}

#[async_trait]
impl PayslipRepository for MySqlPayslipRepository {
    async fn insert(&self, new: &mut NewPayslip) -> Result<Payslip, RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(db_err)?;

        let result = sqlx::query!(
            "insert into payslips (staff_id, pay_year, pay_month, status, finalized_at)
             values (?, ?, ?, ?, if(? = 'finalized', current_timestamp(6), null))",
            new.staff_id().as_i64(),
            new.period().year(),
            new.period().month(),
            encode_status(new.status()),
            encode_status(new.status()),
        )
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;

        // ここで採番される
        let payslip_id = i64::try_from(result.last_insert_id())
            .map_err(corrupted)
            .and_then(|id| PayslipId::from_i64(id).map_err(corrupted))?;

        for line in new.lines() {
            sqlx::query!(
                "insert into payslip_lines (payslip_id, project_id, work_minutes, hourly_rate)
                 values (?, ?, ?, ?)",
                payslip_id.as_i64(),
                line.project_id().as_i64(),
                line.work_minutes().as_minutes(),
                line.hourly_rate().as_yen(),
            )
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }

        // 集約と同じトランザクションでoutboxに書く。新規はここで初めてIDが分かるので、
        // aggregate_id はinfrastructureが埋める(domainのイベントはIDを持たない)
        write_outbox(&mut tx, payslip_id, new.take_events()).await?;

        tx.commit().await.map_err(db_err)?;

        // 採番済みの集約として読み直して返す
        self.find(payslip_id)
            .await?
            .ok_or_else(|| RepositoryError::CorruptedData("inserted row not found".into()))
    }

    async fn update(&self, payslip: &mut Payslip) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(db_err)?;

        sqlx::query!(
            "update payslips
             set status = ?,
                 finalized_at = if(? = 'finalized', coalesce(finalized_at, current_timestamp(6)), null)
             where id = ?",
            encode_status(payslip.status()),
            encode_status(payslip.status()),
            payslip.id().as_i64(),
        )
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;

        // outboxへの書き込みは insert と同様に同じトランザクションで行う
        write_outbox(&mut tx, payslip.id(), payslip.take_events()).await?;

        tx.commit().await.map_err(db_err)
    }

    async fn find(&self, id: PayslipId) -> Result<Option<Payslip>, RepositoryError> {
        let rows = sqlx::query_as!(
            JoinedRow,
            "select p.id, p.staff_id, p.pay_year, p.pay_month, p.status,
                    l.project_id, l.work_minutes, l.hourly_rate
             from payslips p
             join payslip_lines l on l.payslip_id = p.id
             where p.id = ?
             order by l.id",
            id.as_i64(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;

        Ok(assemble(rows)?.pop())
    }

    async fn list_by_staff(&self, staff_id: StaffId) -> Result<Vec<Payslip>, RepositoryError> {
        let rows = sqlx::query_as!(
            JoinedRow,
            "select p.id, p.staff_id, p.pay_year, p.pay_month, p.status,
                    l.project_id, l.work_minutes, l.hourly_rate
             from payslips p
             join payslip_lines l on l.payslip_id = p.id
             where p.staff_id = ? and p.superseded_at is null
             order by p.pay_year desc, p.pay_month desc, p.id, l.id",
            staff_id.as_i64(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;

        assemble(rows)
    }
}

async fn write_outbox(
    tx: &mut Transaction<'_, MySql>,
    payslip_id: PayslipId,
    events: Vec<PayslipEvent>,
) -> Result<(), RepositoryError> {
    for event in events {
        let (event_type, payload) = encode_event(&event);

        sqlx::query!(
            "insert into outbox (aggregate_type, aggregate_id, event_type, payload)
             values ('payslip', ?, ?, ?)",
            payslip_id.as_i64(),
            event_type,
            payload,
        )
        .execute(&mut **tx)
        .await
        .map_err(db_err)?;
    }
    Ok(())
}

/// JOIN の行(明細ごとに id 順で連続している)を集約に組み立てる。
/// DB表現→domainの変換もinfrastructureの責務。想定外の値は壊れたデータとして扱う
fn assemble(rows: Vec<JoinedRow>) -> Result<Vec<Payslip>, RepositoryError> {
    let mut payslips = Vec::new();
    let mut current: Option<(JoinedRow, Vec<PayslipLine>)> = None;

    for row in rows {
        // 値オブジェクトのコンストラクタを再利用して検証し、
        // 壊れたデータから不正な集約が組み立つのを防ぐ
        let line = PayslipLine::new(
            ProjectId::from_i64(row.project_id).map_err(corrupted)?,
            WorkMinutes::from_minutes(row.work_minutes).map_err(corrupted)?,
            Money::from_yen(row.hourly_rate).map_err(corrupted)?,
        );
        match &mut current {
            Some((head, lines)) if head.id == row.id => lines.push(line),
            _ => {
                if let Some((head, lines)) = current.take() {
                    payslips.push(reconstruct(&head, lines)?);
                }
                current = Some((row, vec![line]));
            }
        }
    }
    if let Some((head, lines)) = current {
        payslips.push(reconstruct(&head, lines)?);
    }
    Ok(payslips)
}

fn reconstruct(head: &JoinedRow, lines: Vec<PayslipLine>) -> Result<Payslip, RepositoryError> {
    let status = match head.status.as_str() {
        "draft" => PayslipStatus::Draft,
        "finalized" => PayslipStatus::Finalized,
        other => return Err(RepositoryError::CorruptedData(format!("unknown status: {other}"))),
    };
    Payslip::reconstruct(
        PayslipId::from_i64(head.id).map_err(corrupted)?,
        StaffId::from_i64(head.staff_id).map_err(corrupted)?,
        PayPeriod::new(head.pay_year, head.pay_month).map_err(corrupted)?,
        lines,
        status,
    )
    .map_err(corrupted)
}

fn encode_status(status: PayslipStatus) -> &'static str {
    match status {
        PayslipStatus::Draft => "draft",
        PayslipStatus::Finalized => "finalized",
    }
}

/// イベントを outbox の (`event_type`, `payload`) にする。キューに流す形は infrastructure の DTO
fn encode_event(
    event: &PayslipEvent,
) -> (&'static str, sqlx::types::Json<PayslipFinalizedPayload>) {
    match event {
        PayslipEvent::Finalized { staff_id, period, total } => (
            PayslipFinalizedPayload::EVENT_TYPE,
            sqlx::types::Json(PayslipFinalizedPayload {
                staff_id: staff_id.as_i64(),
                pay_year: period.year(),
                pay_month: period.month(),
                total_yen: total.as_yen(),
            }),
        ),
    }
}
