use async_trait::async_trait;
use payroll_domain::payslip::{
    NewPayslip, PayPeriod, Payslip, PayslipId, PayslipLine, PayslipStatus, WorkMinutes,
};
use payroll_domain::project::ProjectId;
use payroll_domain::staff::StaffId;
use payroll_usecase::ports::repository::{PayslipRepository, RepositoryError};
use platform_kernel::Money;
use sqlx::mysql::MySqlPool;

use crate::db::{corrupted, db_err};
use crate::transaction::MySqlTx;

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
impl PayslipRepository<MySqlTx> for MySqlPayslipRepository {
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

    async fn insert(
        &self,
        tx: &mut MySqlTx,
        new: &NewPayslip,
    ) -> Result<PayslipId, RepositoryError> {
        let result = sqlx::query!(
            "insert into payslips (staff_id, pay_year, pay_month, status, finalized_at)
             values (?, ?, ?, ?, if(? = 'finalized', current_timestamp(6), null))",
            new.staff_id().as_i64(),
            new.period().year(),
            new.period().month(),
            encode_status(new.status()),
            encode_status(new.status()),
        )
        .execute(&mut **tx)
        .await
        .map_err(db_err)?;

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
            .execute(&mut **tx)
            .await
            .map_err(db_err)?;
        }

        Ok(payslip_id)
    }

    async fn update(&self, tx: &mut MySqlTx, payslip: &Payslip) -> Result<(), RepositoryError> {
        sqlx::query!(
            "update payslips
             set status = ?,
                 finalized_at = if(? = 'finalized', coalesce(finalized_at, current_timestamp(6)), null)
             where id = ?",
            encode_status(payslip.status()),
            encode_status(payslip.status()),
            payslip.id().as_i64(),
        )
        .execute(&mut **tx)
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
