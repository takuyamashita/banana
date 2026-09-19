//! 給与(payroll)で登録された派遣社員・案件の写し。
//!
//! 給与から出来事が届いたら記録し、勤務表を書くときに使う。同じ出来事が2回届いても、記録は変わらない

use std::sync::Arc;

use timesheet_domain::project::Project;
use timesheet_domain::staff::Staff;

use crate::UseCaseError;
use crate::ports::database::Database;
use crate::ports::repository::{ProjectRepository, StaffRepository};

/// 給与で派遣社員が登録されたことを記録する
pub struct RecordStaffUseCase {
    staff: Arc<dyn StaffRepository>,
    db: Arc<dyn Database>,
}

impl RecordStaffUseCase {
    #[must_use]
    pub fn new(staff: Arc<dyn StaffRepository>, db: Arc<dyn Database>) -> Self {
        Self { staff, db }
    }

    pub async fn execute(&self, staff: Staff) -> Result<(), UseCaseError> {
        let mut db = self.db.connection().await?;
        Ok(self.staff.save(&mut db, &staff).await?)
    }
}

/// 給与で案件が登録されたことを記録する
pub struct RecordProjectUseCase {
    projects: Arc<dyn ProjectRepository>,
    db: Arc<dyn Database>,
}

impl RecordProjectUseCase {
    #[must_use]
    pub fn new(projects: Arc<dyn ProjectRepository>, db: Arc<dyn Database>) -> Self {
        Self { projects, db }
    }

    pub async fn execute(&self, project: Project) -> Result<(), UseCaseError> {
        let mut db = self.db.connection().await?;
        Ok(self.projects.save(&mut db, &project).await?)
    }
}

/// 派遣社員が、稼働を書ける案件を一覧する(案件番号の順)
pub struct ListProjectsUseCase {
    projects: Arc<dyn ProjectRepository>,
}

impl ListProjectsUseCase {
    #[must_use]
    pub fn new(projects: Arc<dyn ProjectRepository>) -> Self {
        Self { projects }
    }

    pub async fn execute(&self) -> Result<Vec<Project>, UseCaseError> {
        Ok(self.projects.list().await?)
    }
}
