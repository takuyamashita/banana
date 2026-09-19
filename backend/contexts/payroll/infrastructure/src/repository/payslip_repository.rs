use async_trait::async_trait;
use payroll_domain::payslip::{
    FinalizedPayslip, HourlyRate, NewPayslip, PayPeriod, Payslip, PayslipId, PayslipLine,
    WorkMinutes,
};
use payroll_domain::project::ProjectId;
use payroll_domain::staff::StaffId;
use payroll_usecase::ports::database::Db;
use payroll_usecase::ports::repository::{PayslipRepository, RepositoryError};
use sqlx::Connection as _;
use sqlx::mysql::MySqlPool;
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

use crate::database::{mysql, mysql_tx};
use crate::db::{corrupted, db_err};

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
    finalized_at: Option<PrimitiveDateTime>,
    project_id: i64,
    work_minutes: u32,
    hourly_rate: i64,
}

#[async_trait]
impl PayslipRepository for MySqlPayslipRepository {
    async fn find(&self, id: PayslipId) -> Result<Option<Payslip>, RepositoryError> {
        let rows = sqlx::query_as!(
            JoinedRow,
            "select p.id, p.staff_id, p.pay_year, p.pay_month, p.status, p.finalized_at,
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
            "select p.id, p.staff_id, p.pay_year, p.pay_month, p.status, p.finalized_at,
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

    async fn find_for_update(
        &self,
        db: &mut Db,
        id: PayslipId,
    ) -> Result<Option<Payslip>, RepositoryError> {
        let conn = mysql_tx(db)?;
        let rows = sqlx::query_as!(
            JoinedRow,
            "select p.id, p.staff_id, p.pay_year, p.pay_month, p.status, p.finalized_at,
                    l.project_id, l.work_minutes, l.hourly_rate
             from payslips p
             join payslip_lines l on l.payslip_id = p.id
             where p.id = ?
             order by l.id
             for update",
            id.as_i64(),
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(db_err)?;

        Ok(assemble(rows)?.pop())
    }

    async fn insert(&self, db: &mut Db, new: &NewPayslip) -> Result<PayslipId, RepositoryError> {
        // 給与明細と明細行は必ず一緒に書く。渡された書き込み先がトランザクションなら、その中の
        // SAVEPOINT になる
        let mut tx = mysql(db)?.begin().await.map_err(db_err)?;
        // 登録するのは作成中の給与明細だけ(確定は登録してから行う)
        let result = sqlx::query!(
            "insert into payslips (staff_id, pay_year, pay_month, status) values (?, ?, ?, 'draft')",
            new.content().staff_id().as_i64(),
            new.content().period().year(),
            new.content().period().month(),
        )
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;

        let payslip_id = i64::try_from(result.last_insert_id())
            .map_err(corrupted)
            .and_then(|id| PayslipId::from_i64(id).map_err(corrupted))?;

        for line in new.content().lines() {
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

        tx.commit().await.map_err(db_err)?;
        Ok(payslip_id)
    }

    async fn record_finalized(
        &self,
        db: &mut Db,
        payslip: &FinalizedPayslip,
    ) -> Result<(), RepositoryError> {
        let conn = mysql(db)?;
        // 作成中の記録だけを書き換える。ロックせずに読んだ古い作成中を書き戻して、
        // 確定済み(振込の出来事は記録済み)を作成中に戻すことがないようにする
        let result = sqlx::query!(
            "update payslips set status = 'finalized', finalized_at = ? where id = ? and status = 'draft'",
            utc(payslip.finalized_at()),
            payslip.content().id().as_i64(),
        )
        .execute(&mut *conn)
        .await
        .map_err(db_err)?;
        if result.rows_affected() != 1 {
            return Err(RepositoryError::Conflict("作成中の給与明細ではありません".to_owned()));
        }
        Ok(())
    }
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
            HourlyRate::from_yen(row.hourly_rate).map_err(corrupted)?,
        )
        .map_err(corrupted)?;
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

#[allow(clippy::disallowed_methods, reason = "リポジトリ実装は記録から組み立て直す")]
fn reconstruct(head: &JoinedRow, lines: Vec<PayslipLine>) -> Result<Payslip, RepositoryError> {
    let id = PayslipId::from_i64(head.id).map_err(corrupted)?;
    let staff_id = StaffId::from_i64(head.staff_id).map_err(corrupted)?;
    let period = PayPeriod::new(head.pay_year, head.pay_month).map_err(corrupted)?;
    match (head.status.as_str(), head.finalized_at) {
        ("draft", None) => Payslip::reconstruct_draft(id, staff_id, period, lines),
        ("finalized", Some(at)) => {
            Payslip::reconstruct_finalized(id, staff_id, period, lines, at.assume_utc())
        }
        (status, at) => {
            return Err(RepositoryError::CorruptedData(format!(
                "status と finalized_at が合わない: {status}, {at:?}"
            )));
        }
    }
    .map_err(corrupted)
}

fn utc(at: OffsetDateTime) -> PrimitiveDateTime {
    let at = at.to_offset(UtcOffset::UTC);
    PrimitiveDateTime::new(at.date(), at.time())
}
