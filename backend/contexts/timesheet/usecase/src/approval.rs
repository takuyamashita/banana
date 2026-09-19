//! 管理者が、申告された勤務表を確かめて承認するか、理由を付けて差し戻す

use std::sync::Arc;

use timesheet_domain::staff::Staff;
use timesheet_domain::timesheet::{ReturnReason, Timesheet, TimesheetId};

use crate::UseCaseError;
use crate::ports::clock::Clock;
use crate::ports::database::Database;
use crate::ports::events::EventOutbox;
use crate::ports::repository::{StaffRepository, TimesheetRepository};

/// 勤務表と、それを書いた派遣社員
#[derive(Debug)]
pub struct TimesheetView {
    pub timesheet: Timesheet,
    /// 派遣社員(写しが見つからなければ None)
    pub staff: Option<Staff>,
}

async fn view(
    staff: &dyn StaffRepository,
    timesheet: Timesheet,
) -> Result<TimesheetView, UseCaseError> {
    let staff = staff.find(timesheet.content().staff_id()).await?;
    Ok(TimesheetView { timesheet, staff })
}

/// 管理者が、承認を待っている勤務表を申告された順に一覧する
pub struct ListSubmittedTimesheetsUseCase {
    timesheets: Arc<dyn TimesheetRepository>,
    staff: Arc<dyn StaffRepository>,
}

impl ListSubmittedTimesheetsUseCase {
    #[must_use]
    pub fn new(timesheets: Arc<dyn TimesheetRepository>, staff: Arc<dyn StaffRepository>) -> Self {
        Self { timesheets, staff }
    }

    pub async fn execute(&self) -> Result<Vec<TimesheetView>, UseCaseError> {
        let mut views = Vec::new();
        for timesheet in self.timesheets.list_submitted().await? {
            views.push(view(self.staff.as_ref(), timesheet).await?);
        }
        Ok(views)
    }
}

/// 管理者が、申告された勤務表を承認する。
///
/// 承認するとその月の稼働が決まり、案件ごとの稼働の合計を給与に知らせる。
/// 承認と、承認したという出来事は一緒に記録する(どちらか一方だけが残ることはない)
pub struct ApproveTimesheetUseCase {
    timesheets: Arc<dyn TimesheetRepository>,
    staff: Arc<dyn StaffRepository>,
    outbox: Arc<dyn EventOutbox>,
    db: Arc<dyn Database>,
    clock: Arc<dyn Clock>,
}

impl ApproveTimesheetUseCase {
    #[must_use]
    pub fn new(
        timesheets: Arc<dyn TimesheetRepository>,
        staff: Arc<dyn StaffRepository>,
        outbox: Arc<dyn EventOutbox>,
        db: Arc<dyn Database>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { timesheets, staff, outbox, db, clock }
    }

    pub async fn execute(&self, id: TimesheetId) -> Result<TimesheetView, UseCaseError> {
        let mut tx = self.db.transaction().await?;
        let (approved, event) = match self.timesheets.find_for_update(&mut tx, id).await? {
            Some(Timesheet::Submitted(submitted)) => submitted.approve(self.clock.now()),
            Some(_) => {
                return Err(UseCaseError::FailedPrecondition(
                    "承認できるのは申告された勤務表だけです".into(),
                ));
            }
            None => return Err(UseCaseError::NotFound),
        };
        let approved = Timesheet::from(approved);
        self.timesheets.update(&mut tx, &approved).await?;
        self.outbox.append(&mut tx, event).await?;
        tx.commit().await?;
        view(self.staff.as_ref(), approved).await
    }
}

/// 管理者が、申告された勤務表を理由を付けて差し戻す。勤務表は作成中に戻り、派遣社員が直して申告し直す
pub struct ReturnTimesheetUseCase {
    timesheets: Arc<dyn TimesheetRepository>,
    staff: Arc<dyn StaffRepository>,
    db: Arc<dyn Database>,
}

impl ReturnTimesheetUseCase {
    #[must_use]
    pub fn new(
        timesheets: Arc<dyn TimesheetRepository>,
        staff: Arc<dyn StaffRepository>,
        db: Arc<dyn Database>,
    ) -> Self {
        Self { timesheets, staff, db }
    }

    pub async fn execute(
        &self,
        id: TimesheetId,
        reason: ReturnReason,
    ) -> Result<TimesheetView, UseCaseError> {
        let mut tx = self.db.transaction().await?;
        let returned = match self.timesheets.find_for_update(&mut tx, id).await? {
            Some(Timesheet::Submitted(submitted)) => Timesheet::from(submitted.send_back(reason)),
            Some(_) => {
                return Err(UseCaseError::FailedPrecondition(
                    "差し戻せるのは申告された勤務表だけです".into(),
                ));
            }
            None => return Err(UseCaseError::NotFound),
        };
        self.timesheets.update(&mut tx, &returned).await?;
        tx.commit().await?;
        view(self.staff.as_ref(), returned).await
    }
}
