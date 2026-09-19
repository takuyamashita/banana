use async_trait::async_trait;
use payroll_domain::payslip::{
    NewPayslip, PayPeriod, Payslip, PayslipId, PayslipLine, PayslipStatus, WorkMinutes,
};
use payroll_domain::project::ProjectId;
use payroll_domain::staff::StaffId;
use payroll_usecase::ports::database::Db;
use payroll_usecase::ports::repository::{PayslipRepository, RepositoryError};
use platform_kernel::Money;
use sqlx::Connection as _;
use sqlx::mysql::MySqlPool;
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

use crate::database::mysql;
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

    async fn insert(&self, db: &mut Db, new: &NewPayslip) -> Result<PayslipId, RepositoryError> {
        // 給与明細と明細行は必ず一緒に書く。渡された書き込み先がトランザクションなら、その中の
        // SAVEPOINT になる
        let mut tx = mysql(db)?.begin().await.map_err(db_err)?;
        let result = sqlx::query!(
            "insert into payslips (staff_id, pay_year, pay_month, status, finalized_at)
             values (?, ?, ?, ?, ?)",
            new.content().staff_id().as_i64(),
            new.content().period().year(),
            new.content().period().month(),
            encode_status(new.status()),
            finalized_at(new),
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

    async fn update(&self, db: &mut Db, payslip: &Payslip) -> Result<(), RepositoryError> {
        let conn = mysql(db)?;
        sqlx::query!(
            "update payslips set status = ?, finalized_at = ? where id = ?",
            encode_status(payslip.status()),
            finalized_at(payslip),
            payslip.content().id().as_i64(),
        )
        .execute(&mut *conn)
        .await
        .map_err(db_err)?;
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

/// 確定済みなら確定日時を返す。DB の datetime は UTC で持つ(接続の time_zone も UTC)
fn finalized_at<Id>(payslip: &Payslip<Id>) -> Option<PrimitiveDateTime> {
    match payslip {
        Payslip::Draft(_) => None,
        Payslip::Finalized(p) => Some(utc(p.finalized_at())),
    }
}

fn utc(at: OffsetDateTime) -> PrimitiveDateTime {
    let at = at.to_offset(UtcOffset::UTC);
    PrimitiveDateTime::new(at.date(), at.time())
}

fn encode_status(status: PayslipStatus) -> &'static str {
    match status {
        PayslipStatus::Draft => "draft",
        PayslipStatus::Finalized => "finalized",
    }
}
