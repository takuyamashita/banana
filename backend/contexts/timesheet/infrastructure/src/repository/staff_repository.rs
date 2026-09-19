use async_trait::async_trait;
use platform_kernel::UserId;
use sqlx::mysql::MySqlPool;
use timesheet_domain::staff::{Staff, StaffId, StaffName};
use timesheet_usecase::ports::database::Db;
use timesheet_usecase::ports::repository::{RepositoryError, StaffRepository};

use crate::database::mysql;
use crate::db::{corrupted, db_err};

pub struct MySqlStaffRepository {
    pool: MySqlPool,
}

impl MySqlStaffRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

struct Row {
    id: i64,
    user_id: String,
    name: String,
}

fn to_staff(row: Row) -> Result<Staff, RepositoryError> {
    Ok(Staff::new(
        StaffId::from_i64(row.id).map_err(corrupted)?,
        UserId::parse(row.user_id).map_err(corrupted)?,
        StaffName::new(row.name).map_err(corrupted)?,
    ))
}

#[async_trait]
impl StaffRepository for MySqlStaffRepository {
    async fn find(&self, id: StaffId) -> Result<Option<Staff>, RepositoryError> {
        sqlx::query_as!(Row, "select id, user_id, name from staff where id = ?", id.as_i64())
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)?
            .map(to_staff)
            .transpose()
    }

    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Option<Staff>, RepositoryError> {
        sqlx::query_as!(
            Row,
            "select id, user_id, name from staff where user_id = ?",
            user_id.as_str()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_err)?
        .map(to_staff)
        .transpose()
    }

    async fn save(&self, db: &mut Db, staff: &Staff) -> Result<(), RepositoryError> {
        // 届いた内容に置き換える。同じ出来事が2回届いても、記録は変わらない
        sqlx::query!(
            "insert into staff (id, user_id, name) values (?, ?, ?) as new
             on duplicate key update user_id = new.user_id, name = new.name",
            staff.id().as_i64(),
            staff.user_id().as_str(),
            staff.name().as_str(),
        )
        .execute(&mut *mysql(db)?)
        .await
        .map_err(db_err)?;
        Ok(())
    }
}
