use async_trait::async_trait;
use payroll_domain::payout::{NewPayout, Payout, PayoutId, PayoutOutcome};
use payroll_domain::payslip::PayslipId;
use payroll_domain::staff::StaffId;
use payroll_usecase::ports::database::Db;
use payroll_usecase::ports::repository::{PayoutRepository, RepositoryError};
use platform_kernel::Money;
use sqlx::mysql::MySqlPool;

use crate::database::mysql;
use crate::db::{corrupted, db_err, ensure_updated};

/// 振込先から返る理由は長さが決まっていないので、列に収まる長さで切る
const MAX_REASON_CHARS: usize = 1000;

pub struct MySqlPayoutRepository {
    pool: MySqlPool,
}

impl MySqlPayoutRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

struct PayoutRow {
    id: i64,
    payslip_id: i64,
    staff_id: i64,
    amount_yen: i64,
    outcome: String,
    receipt: Option<String>,
    reject_reason: Option<String>,
}

impl TryFrom<PayoutRow> for Payout {
    type Error = RepositoryError;

    #[allow(clippy::disallowed_methods, reason = "リポジトリ実装は記録から組み立て直す")]
    fn try_from(row: PayoutRow) -> Result<Self, Self::Error> {
        let outcome = match (row.outcome.as_str(), row.receipt, row.reject_reason) {
            ("accepted", Some(receipt), None) => PayoutOutcome::Accepted { receipt },
            ("rejected", None, Some(reason)) => PayoutOutcome::Rejected { reason },
            (outcome, ..) => {
                return Err(RepositoryError::CorruptedData(format!(
                    "payout {}: outcome {outcome} does not match receipt/reason",
                    row.id
                )));
            }
        };
        Ok(Payout::reconstruct(
            PayoutId::from_i64(row.id).map_err(corrupted)?,
            PayslipId::from_i64(row.payslip_id).map_err(corrupted)?,
            StaffId::from_i64(row.staff_id).map_err(corrupted)?,
            Money::from_yen(row.amount_yen).map_err(corrupted)?,
            outcome,
        ))
    }
}

#[async_trait]
impl PayoutRepository for MySqlPayoutRepository {
    async fn find_by_payslip(
        &self,
        payslip_id: PayslipId,
    ) -> Result<Option<Payout>, RepositoryError> {
        sqlx::query_as!(
            PayoutRow,
            "select id, payslip_id, staff_id, amount_yen, outcome, receipt, reject_reason
             from payouts where payslip_id = ?",
            payslip_id.as_i64(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?
        .map(Payout::try_from)
        .transpose()
    }

    async fn insert(&self, db: &mut Db, new: &NewPayout) -> Result<PayoutId, RepositoryError> {
        let conn = mysql(db)?;
        let (outcome, receipt, reason) = encode_outcome(new.outcome());
        let result = sqlx::query!(
            "insert into payouts (payslip_id, staff_id, amount_yen, outcome, receipt, reject_reason)
             values (?, ?, ?, ?, ?, ?)",
            new.payslip_id().as_i64(),
            new.staff_id().as_i64(),
            new.amount().as_yen(),
            outcome,
            receipt,
            reason,
        )
        .execute(&mut *conn)
        .await
        .map_err(db_err)?;

        i64::try_from(result.last_insert_id())
            .map_err(corrupted)
            .and_then(|id| PayoutId::from_i64(id).map_err(corrupted))
    }

    async fn update(&self, db: &mut Db, payout: &Payout) -> Result<(), RepositoryError> {
        let conn = mysql(db)?;
        let (outcome, receipt, reason) = encode_outcome(payout.outcome());
        let result = sqlx::query!(
            "update payouts set payslip_id = ?, staff_id = ?, amount_yen = ?, outcome = ?, receipt = ?,
                    reject_reason = ?
             where id = ?",
            payout.payslip_id().as_i64(),
            payout.staff_id().as_i64(),
            payout.amount().as_yen(),
            outcome,
            receipt,
            reason,
            payout.id().as_i64(),
        )
        .execute(&mut *conn)
        .await
        .map_err(db_err)?;
        ensure_updated(result.rows_affected(), "振込依頼", payout.id().as_i64())
    }
}

/// 振込の結果を列(outcome・receipt・reject_reason)に分ける
fn encode_outcome(outcome: &PayoutOutcome) -> (&'static str, Option<&str>, Option<String>) {
    match outcome {
        PayoutOutcome::Accepted { receipt } => ("accepted", Some(receipt.as_str()), None),
        PayoutOutcome::Rejected { reason } => {
            ("rejected", None, Some(reason.chars().take(MAX_REASON_CHARS).collect()))
        }
    }
}
