use time::Date;

use super::WorkMinutes;
use crate::project::ProjectId;

/// 勤務表の1行。ある日に、ある案件で稼働した時間
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkEntry {
    /// 稼働した日
    date: Date,
    /// 稼働した案件
    project_id: ProjectId,
    /// 稼働した時間
    minutes: WorkMinutes,
}

impl WorkEntry {
    #[must_use]
    pub fn new(date: Date, project_id: ProjectId, minutes: WorkMinutes) -> Self {
        Self { date, project_id, minutes }
    }

    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    #[must_use]
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }

    #[must_use]
    pub fn minutes(&self) -> WorkMinutes {
        self.minutes
    }
}
