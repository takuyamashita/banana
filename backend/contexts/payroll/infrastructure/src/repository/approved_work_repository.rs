use async_trait::async_trait;
use payroll_domain::payslip::{PayPeriod, WorkMinutes};
use payroll_domain::project::ProjectId;
use payroll_domain::staff::StaffId;
use payroll_domain::work::{ApprovedWork, ProjectWork};
use payroll_usecase::ports::database::Db;
use payroll_usecase::ports::repository::{ApprovedWorkRepository, RepositoryError};
use sqlx::Connection as _;
use sqlx::mysql::MySqlPool;

use crate::database::mysql;
use crate::db::{corrupted, db_err};

pub struct MySqlApprovedWorkRepository {
    pool: MySqlPool,
}

impl MySqlApprovedWorkRepository {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApprovedWorkRepository for MySqlApprovedWorkRepository {
    async fn find(
        &self,
        staff_id: StaffId,
        period: PayPeriod,
    ) -> Result<Option<ApprovedWork>, RepositoryError> {
        let rows = sqlx::query!(
            "select project_id, work_minutes from approved_work
             where staff_id = ? and work_year = ? and work_month = ?
             order by project_id",
            staff_id.as_i64(),
            period.year(),
            period.month(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_err)?;
        if rows.is_empty() {
            return Ok(None);
        }
        let projects = rows
            .into_iter()
            .map(|r| {
                Ok(ProjectWork {
                    project_id: ProjectId::from_i64(r.project_id).map_err(corrupted)?,
                    minutes: WorkMinutes::from_minutes(r.work_minutes).map_err(corrupted)?,
                })
            })
            .collect::<Result<Vec<_>, RepositoryError>>()?;
        ApprovedWork::new(staff_id, period, projects).map(Some).map_err(corrupted)
    }

    async fn save(&self, db: &mut Db, work: &ApprovedWork) -> Result<(), RepositoryError> {
        // 派遣社員・月ごとに丸ごと置き換える。渡された書き込み先がトランザクションなら、その中の SAVEPOINT になる
        let mut tx = mysql(db)?.begin().await.map_err(db_err)?;
        let (staff, year, month) =
            (work.staff_id().as_i64(), work.period().year(), work.period().month());
        sqlx::query!(
            "delete from approved_work where staff_id = ? and work_year = ? and work_month = ?",
            staff,
            year,
            month,
        )
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;
        for project in work.projects() {
            sqlx::query!(
                "insert into approved_work (staff_id, work_year, work_month, project_id, work_minutes)
                 values (?, ?, ?, ?, ?)",
                staff,
                year,
                month,
                project.project_id.as_i64(),
                project.minutes.as_minutes(),
            )
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;
        }
        tx.commit().await.map_err(db_err)?;
        Ok(())
    }
}
