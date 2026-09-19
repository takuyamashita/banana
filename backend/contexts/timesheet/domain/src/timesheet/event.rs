use time::OffsetDateTime;

use super::{TimesheetId, WorkMonth};
use crate::project::ProjectId;
use crate::staff::StaffId;

/// 勤務表に起きた、他の業務が知るべき出来事
#[must_use = "勤務表の出来事は記録して他の業務に知らせる必要がある"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimesheetEvent {
    /// 勤務表が承認され、その月の案件ごとの稼働が決まった。給与の計算はこれを元にする
    Approved {
        timesheet_id: TimesheetId,
        staff_id: StaffId,
        month: WorkMonth,
        /// 案件ごとの稼働の合計。案件番号の順
        work: Vec<ProjectWork>,
        /// 承認した日時
        approved_at: OffsetDateTime,
    },
}

/// 1つの案件での、1か月の稼働の合計
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectWork {
    pub project_id: ProjectId,
    /// 稼働の合計(分)
    pub minutes: u32,
}
