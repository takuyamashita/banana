//! 派遣社員が、自分の月の勤務表を書いて申告する

use std::sync::Arc;

use platform_kernel::AuthenticatedUser;
use timesheet_domain::project::ProjectId;
use timesheet_domain::staff::Staff;
use timesheet_domain::timesheet::{Timesheet, WorkEntry, WorkMonth};

use crate::UseCaseError;
use crate::ports::clock::Clock;
use crate::ports::database::Database;
use crate::ports::repository::{ProjectRepository, StaffRepository, TimesheetRepository};

/// 派遣社員のその月の勤務表
#[derive(Debug)]
pub enum MyTimesheet {
    /// まだ書き始めていない(稼働が1件もない作成中の勤務表として見せる)
    NotStarted { staff: Staff, month: WorkMonth },
    /// 書き始めている
    Started { staff: Staff, timesheet: Timesheet },
}

/// ログインした本人がどの派遣社員かを知る。給与で登録された派遣社員が、まだ勤怠に届いていないこともある
async fn me(staff: &dyn StaffRepository, user: &AuthenticatedUser) -> Result<Staff, UseCaseError> {
    staff.find_by_user_id(&user.user_id).await?.ok_or_else(|| {
        UseCaseError::FailedPrecondition(
            "派遣社員の登録がまだ勤怠に届いていません。しばらくしてから開き直してください".into(),
        )
    })
}

/// 派遣社員が、自分のその月の勤務表を見る
pub struct GetMyTimesheetUseCase {
    timesheets: Arc<dyn TimesheetRepository>,
    staff: Arc<dyn StaffRepository>,
}

impl GetMyTimesheetUseCase {
    #[must_use]
    pub fn new(timesheets: Arc<dyn TimesheetRepository>, staff: Arc<dyn StaffRepository>) -> Self {
        Self { timesheets, staff }
    }

    pub async fn execute(
        &self,
        user: &AuthenticatedUser,
        month: WorkMonth,
    ) -> Result<MyTimesheet, UseCaseError> {
        let staff = me(self.staff.as_ref(), user).await?;
        Ok(match self.timesheets.find_by_staff_month(staff.id(), month).await? {
            Some(timesheet) => MyTimesheet::Started { staff, timesheet },
            None => MyTimesheet::NotStarted { staff, month },
        })
    }
}

/// 派遣社員が、自分のその月の勤務表の稼働を書く(書いてあった行は、渡した行で置き換わる)。
///
/// 書けるのは作成中の勤務表だけ。まだ書き始めていなければ、ここで書き始める。
/// 稼働を書けるのは、勤怠に届いている(給与で登録された)案件だけ
pub struct SaveMyTimesheetUseCase {
    timesheets: Arc<dyn TimesheetRepository>,
    staff: Arc<dyn StaffRepository>,
    projects: Arc<dyn ProjectRepository>,
    db: Arc<dyn Database>,
}

impl SaveMyTimesheetUseCase {
    #[must_use]
    pub fn new(
        timesheets: Arc<dyn TimesheetRepository>,
        staff: Arc<dyn StaffRepository>,
        projects: Arc<dyn ProjectRepository>,
        db: Arc<dyn Database>,
    ) -> Self {
        Self { timesheets, staff, projects, db }
    }

    pub async fn execute(
        &self,
        user: &AuthenticatedUser,
        month: WorkMonth,
        entries: Vec<WorkEntry>,
    ) -> Result<Timesheet, UseCaseError> {
        let staff = me(self.staff.as_ref(), user).await?;
        self.ensure_known_projects(&entries).await?;

        let mut tx = self.db.transaction().await?;
        let saved = match self.timesheets.find_by_staff_month(staff.id(), month).await? {
            None => {
                let new = Timesheet::start(staff.id(), month).record(entries)?;
                let id = self.timesheets.insert(&mut tx, &new).await?;
                self.timesheets.find_for_update(&mut tx, id).await?
            }
            Some(existing) => {
                let id = existing.content().id();
                let Some(Timesheet::Draft(draft)) =
                    self.timesheets.find_for_update(&mut tx, id).await?
                else {
                    return Err(UseCaseError::FailedPrecondition(
                        "申告した勤務表は書き直せません".into(),
                    ));
                };
                let updated = Timesheet::from(draft.record(entries)?);
                self.timesheets.update(&mut tx, &updated).await?;
                Some(updated)
            }
        };
        tx.commit().await?;
        saved.ok_or_else(|| UseCaseError::Internal("記録した勤務表が見つかりません".into()))
    }

    async fn ensure_known_projects(&self, entries: &[WorkEntry]) -> Result<(), UseCaseError> {
        let mut checked: Vec<ProjectId> = Vec::new();
        for entry in entries {
            let id = entry.project_id();
            if checked.contains(&id) {
                continue;
            }
            if self.projects.find(id).await?.is_none() {
                return Err(UseCaseError::InvalidInput(format!(
                    "案件 #{} は登録されていません",
                    id.as_i64()
                )));
            }
            checked.push(id);
        }
        Ok(())
    }
}

/// 派遣社員が、書き終えた自分のその月の勤務表を申告する。申告すると書き直せず、管理者の承認を待つ
pub struct SubmitMyTimesheetUseCase {
    timesheets: Arc<dyn TimesheetRepository>,
    staff: Arc<dyn StaffRepository>,
    db: Arc<dyn Database>,
    clock: Arc<dyn Clock>,
}

impl SubmitMyTimesheetUseCase {
    #[must_use]
    pub fn new(
        timesheets: Arc<dyn TimesheetRepository>,
        staff: Arc<dyn StaffRepository>,
        db: Arc<dyn Database>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { timesheets, staff, db, clock }
    }

    pub async fn execute(
        &self,
        user: &AuthenticatedUser,
        month: WorkMonth,
    ) -> Result<Timesheet, UseCaseError> {
        let staff = me(self.staff.as_ref(), user).await?;
        let Some(existing) = self.timesheets.find_by_staff_month(staff.id(), month).await? else {
            return Err(UseCaseError::FailedPrecondition(
                "稼働が1件もない勤務表は申告できません".into(),
            ));
        };

        let mut tx = self.db.transaction().await?;
        let submitted =
            match self.timesheets.find_for_update(&mut tx, existing.content().id()).await? {
                Some(Timesheet::Draft(draft)) => Timesheet::from(draft.submit(self.clock.now())?),
                Some(_) => {
                    return Err(UseCaseError::FailedPrecondition(
                        "この勤務表は申告済みです".into(),
                    ));
                }
                None => return Err(UseCaseError::NotFound),
            };
        self.timesheets.update(&mut tx, &submitted).await?;
        tx.commit().await?;
        Ok(submitted)
    }
}
