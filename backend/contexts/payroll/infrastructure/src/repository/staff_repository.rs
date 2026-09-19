use async_trait::async_trait;
use payroll_domain::staff::{DisplayName, NewStaff, Staff, StaffId};
use payroll_usecase::ports::repository::{RepositoryError, StaffRepository};
use platform_kernel::{Email, UserId};
use sqlx::mysql::MySqlPool;

use crate::db::{corrupted, db_err};
use crate::transaction::MySqlTx;

pub struct MySqlStaffRepository {
    pool: MySqlPool,
}

impl MySqlStaffRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

struct StaffRow {
    id: i64,
    user_id: String,
    email: String,
    display_name: String,
}

impl TryFrom<StaffRow> for Staff {
    type Error = RepositoryError;

    fn try_from(row: StaffRow) -> Result<Self, Self::Error> {
        Ok(Staff::reconstruct(
            StaffId::from_i64(row.id).map_err(corrupted)?,
            UserId::parse(row.user_id).map_err(corrupted)?,
            Email::parse(row.email).map_err(corrupted)?,
            DisplayName::new(row.display_name).map_err(corrupted)?,
        ))
    }
}

#[async_trait]
impl StaffRepository<MySqlTx> for MySqlStaffRepository {
    async fn find(&self, id: StaffId) -> Result<Option<Staff>, RepositoryError> {
        sqlx::query_as!(
            StaffRow,
            "select id, user_id, email, display_name from staff where id = ?",
            id.as_i64(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?
        .map(Staff::try_from)
        .transpose()
    }

    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Option<Staff>, RepositoryError> {
        sqlx::query_as!(
            StaffRow,
            "select id, user_id, email, display_name from staff where user_id = ?",
            user_id.as_str(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?
        .map(Staff::try_from)
        .transpose()
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<Staff>, RepositoryError> {
        sqlx::query_as!(
            StaffRow,
            "select id, user_id, email, display_name from staff where email = ?",
            email.as_str(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?
        .map(Staff::try_from)
        .transpose()
    }

    async fn insert(&self, tx: &mut MySqlTx, new: &NewStaff) -> Result<StaffId, RepositoryError> {
        let result = sqlx::query!(
            "insert into staff (user_id, email, display_name) values (?, ?, ?)",
            new.user_id().as_str(),
            new.email().as_str(),
            new.display_name().as_str(),
        )
        .execute(&mut **tx)
        .await
        .map_err(db_err)?;

        i64::try_from(result.last_insert_id())
            .map_err(corrupted)
            .and_then(|id| StaffId::from_i64(id).map_err(corrupted))
    }
}
