use async_trait::async_trait;
use sqlx::Connection as _;
use sqlx::mysql::{MySqlConnection, MySqlPool};
use time::{Date, PrimitiveDateTime};
use timesheet_domain::project::ProjectId;
use timesheet_domain::staff::StaffId;
use timesheet_domain::timesheet::{
    NewTimesheet, ReturnReason, Timesheet, TimesheetId, TimesheetStatus, WorkEntry, WorkMinutes,
    WorkMonth,
};
use timesheet_usecase::ports::database::Db;
use timesheet_usecase::ports::repository::{RepositoryError, TimesheetRepository};

use crate::database::{mysql, mysql_tx};
use crate::db::{corrupted, db_err, ensure_updated, to_db, utc};

pub struct MySqlTimesheetRepository {
    pool: MySqlPool,
}

impl MySqlTimesheetRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

/// 勤務表の1行
struct Row {
    id: i64,
    staff_id: i64,
    work_year: u16,
    work_month: u8,
    status: String,
    returned_reason: Option<String>,
    submitted_at: Option<PrimitiveDateTime>,
    approved_at: Option<PrimitiveDateTime>,
}

/// 稼働の行
struct EntryRow {
    work_date: Date,
    project_id: i64,
    work_minutes: i32,
}

fn status_code(status: TimesheetStatus) -> &'static str {
    match status {
        TimesheetStatus::Draft => "draft",
        TimesheetStatus::Submitted => "submitted",
        TimesheetStatus::Approved => "approved",
    }
}

async fn entries_of(
    conn: &mut MySqlConnection,
    id: i64,
) -> Result<Vec<WorkEntry>, RepositoryError> {
    sqlx::query_as!(
        EntryRow,
        "select work_date, project_id, work_minutes from timesheet_entries
         where timesheet_id = ? order by work_date, project_id",
        id,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(db_err)?
    .into_iter()
    .map(|e| {
        Ok(WorkEntry::new(
            e.work_date,
            ProjectId::from_i64(e.project_id).map_err(corrupted)?,
            WorkMinutes::from_minutes(u32::try_from(e.work_minutes).map_err(corrupted)?)
                .map_err(corrupted)?,
        ))
    })
    .collect()
}

/// 記録されている内容から勤務表を組み立て直す。状態と日時が食い違えば壊れたデータとして報告する
#[allow(clippy::disallowed_methods, reason = "リポジトリ実装は記録から組み立て直す")]
fn assemble(row: Row, entries: Vec<WorkEntry>) -> Result<Timesheet, RepositoryError> {
    let id = TimesheetId::from_i64(row.id).map_err(corrupted)?;
    let staff_id = StaffId::from_i64(row.staff_id).map_err(corrupted)?;
    let month = WorkMonth::new(row.work_year, row.work_month).map_err(corrupted)?;
    let missing = |what: &str| corrupted(format!("勤務表 {} の{what}がありません", row.id));
    let timesheet = match row.status.as_str() {
        "draft" => {
            let returned =
                row.returned_reason.map(ReturnReason::new).transpose().map_err(corrupted)?;
            Timesheet::reconstruct_draft(id, staff_id, month, entries, returned)
        }
        "submitted" => {
            let submitted = utc(row.submitted_at.ok_or_else(|| missing("申告日時"))?);
            Timesheet::reconstruct_submitted(id, staff_id, month, entries, submitted)
        }
        "approved" => {
            let submitted = utc(row.submitted_at.ok_or_else(|| missing("申告日時"))?);
            let approved = utc(row.approved_at.ok_or_else(|| missing("承認日時"))?);
            Timesheet::reconstruct_approved(id, staff_id, month, entries, submitted, approved)
        }
        other => return Err(corrupted(format!("勤務表 {} の状態が不明です: {other}", row.id))),
    };
    timesheet.map_err(corrupted)
}

async fn load(
    conn: &mut MySqlConnection,
    rows: Vec<Row>,
) -> Result<Vec<Timesheet>, RepositoryError> {
    let mut timesheets = Vec::with_capacity(rows.len());
    for row in rows {
        let entries = entries_of(conn, row.id).await?;
        timesheets.push(assemble(row, entries)?);
    }
    Ok(timesheets)
}

/// 稼働の行を書き込む(勤務表の行を消してから入れ直す)
async fn write_entries(
    conn: &mut MySqlConnection,
    id: i64,
    entries: &[WorkEntry],
) -> Result<(), RepositoryError> {
    sqlx::query!("delete from timesheet_entries where timesheet_id = ?", id)
        .execute(&mut *conn)
        .await
        .map_err(db_err)?;
    for entry in entries {
        sqlx::query!(
            "insert into timesheet_entries (timesheet_id, work_date, project_id, work_minutes)
             values (?, ?, ?, ?)",
            id,
            entry.date(),
            entry.project_id().as_i64(),
            entry.minutes().as_minutes(),
        )
        .execute(&mut *conn)
        .await
        .map_err(db_err)?;
    }
    Ok(())
}

#[async_trait]
impl TimesheetRepository for MySqlTimesheetRepository {
    async fn find(&self, id: TimesheetId) -> Result<Option<Timesheet>, RepositoryError> {
        let mut conn = self.pool.acquire().await.map_err(db_err)?;
        let rows = sqlx::query_as!(
            Row,
            "select id, staff_id, work_year, work_month, status, returned_reason, submitted_at, approved_at
             from timesheets where id = ?",
            id.as_i64(),
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(db_err)?;
        Ok(load(&mut conn, rows).await?.pop())
    }

    async fn find_by_staff_month(
        &self,
        staff_id: StaffId,
        month: WorkMonth,
    ) -> Result<Option<Timesheet>, RepositoryError> {
        let mut conn = self.pool.acquire().await.map_err(db_err)?;
        let rows = sqlx::query_as!(
            Row,
            "select id, staff_id, work_year, work_month, status, returned_reason, submitted_at, approved_at
             from timesheets where staff_id = ? and work_year = ? and work_month = ?",
            staff_id.as_i64(),
            month.year(),
            month.month(),
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(db_err)?;
        Ok(load(&mut conn, rows).await?.pop())
    }

    async fn list_submitted(&self) -> Result<Vec<Timesheet>, RepositoryError> {
        let mut conn = self.pool.acquire().await.map_err(db_err)?;
        let rows = sqlx::query_as!(
            Row,
            "select id, staff_id, work_year, work_month, status, returned_reason, submitted_at, approved_at
             from timesheets where status = 'submitted' order by submitted_at, id",
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(db_err)?;
        load(&mut conn, rows).await
    }

    async fn find_for_update(
        &self,
        db: &mut Db,
        id: TimesheetId,
    ) -> Result<Option<Timesheet>, RepositoryError> {
        let conn = mysql_tx(db)?;
        let rows = sqlx::query_as!(
            Row,
            "select id, staff_id, work_year, work_month, status, returned_reason, submitted_at, approved_at
             from timesheets where id = ? for update",
            id.as_i64(),
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(db_err)?;
        Ok(load(conn, rows).await?.pop())
    }

    async fn insert(
        &self,
        db: &mut Db,
        new: &NewTimesheet,
    ) -> Result<TimesheetId, RepositoryError> {
        // 勤務表と稼働の行は必ず一緒に書く。渡された書き込み先がトランザクションなら、その中の SAVEPOINT になる
        let mut tx = mysql(db)?.begin().await.map_err(db_err)?;
        let content = new.content();
        let result = sqlx::query!(
            "insert into timesheets (staff_id, work_year, work_month, status, returned_reason)
             values (?, ?, ?, 'draft', ?)",
            content.staff_id().as_i64(),
            content.month().year(),
            content.month().month(),
            new.returned_reason().map(ReturnReason::as_str),
        )
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;
        let id = i64::try_from(result.last_insert_id()).map_err(corrupted)?;
        write_entries(&mut tx, id, content.entries()).await?;
        tx.commit().await.map_err(db_err)?;
        TimesheetId::from_i64(id).map_err(corrupted)
    }

    async fn update(&self, db: &mut Db, timesheet: &Timesheet) -> Result<(), RepositoryError> {
        let mut tx = mysql(db)?.begin().await.map_err(db_err)?;
        let id = timesheet.content().id().as_i64();
        let (returned, submitted_at, approved_at) = match timesheet {
            Timesheet::Draft(t) => (t.returned_reason().map(ReturnReason::as_str), None, None),
            Timesheet::Submitted(t) => (None, Some(to_db(t.submitted_at())), None),
            Timesheet::Approved(t) => {
                (None, Some(to_db(t.submitted_at())), Some(to_db(t.approved_at())))
            }
        };
        let result = sqlx::query!(
            "update timesheets set status = ?, returned_reason = ?, submitted_at = ?, approved_at = ?
             where id = ?",
            status_code(timesheet.status()),
            returned,
            submitted_at,
            approved_at,
            id,
        )
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;
        ensure_updated(result.rows_affected(), "勤務表", id)?;
        write_entries(&mut tx, id, timesheet.content().entries()).await?;
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }
}
