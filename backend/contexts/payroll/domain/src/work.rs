//! 承認された稼働(work)。
//!
//! 派遣社員が勤怠で申告し、管理者が承認した、ある月の案件ごとの稼働の合計。給与明細の明細行の
//! 稼働の元になる。勤怠(timesheet)で決まったものの写しで、給与では変えない

use std::collections::HashSet;

use thiserror::Error;

use crate::payslip::{PayPeriod, WorkMinutes};
use crate::project::ProjectId;
use crate::staff::StaffId;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WorkError {
    #[error("承認された稼働に案件がありません")]
    Empty,
    #[error("承認された稼働に同じ案件が2回あります")]
    DuplicateProject,
}

/// 1つの案件での、1か月の稼働の合計
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectWork {
    pub project_id: ProjectId,
    pub minutes: WorkMinutes,
}

/// 派遣社員のある月の、承認された案件ごとの稼働
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedWork {
    staff_id: StaffId,
    period: PayPeriod,
    /// 案件ごとの稼働。案件は1回ずつ
    projects: Vec<ProjectWork>,
}

impl ApprovedWork {
    pub fn new(
        staff_id: StaffId,
        period: PayPeriod,
        projects: Vec<ProjectWork>,
    ) -> Result<Self, WorkError> {
        if projects.is_empty() {
            return Err(WorkError::Empty);
        }
        let mut seen = HashSet::new();
        if !projects.iter().all(|p| seen.insert(p.project_id)) {
            return Err(WorkError::DuplicateProject);
        }
        Ok(Self { staff_id, period, projects })
    }

    #[must_use]
    pub fn staff_id(&self) -> StaffId {
        self.staff_id
    }

    #[must_use]
    pub fn period(&self) -> PayPeriod {
        self.period
    }

    #[must_use]
    pub fn projects(&self) -> &[ProjectWork] {
        &self.projects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work(project: i64, minutes: u32) -> ProjectWork {
        ProjectWork {
            project_id: ProjectId::from_i64(project).unwrap(),
            minutes: WorkMinutes::from_minutes(minutes).unwrap(),
        }
    }

    fn staff() -> StaffId {
        StaffId::from_i64(1).unwrap()
    }

    fn september() -> PayPeriod {
        PayPeriod::new(2026, 9).unwrap()
    }

    #[test]
    fn approved_work_has_each_project_once() {
        assert_eq!(ApprovedWork::new(staff(), september(), vec![]), Err(WorkError::Empty));
        assert_eq!(
            ApprovedWork::new(staff(), september(), vec![work(1, 60), work(1, 60)]),
            Err(WorkError::DuplicateProject)
        );
        assert_eq!(
            ApprovedWork::new(staff(), september(), vec![work(1, 60), work(2, 90)])
                .unwrap()
                .projects()
                .len(),
            2
        );
    }
}
