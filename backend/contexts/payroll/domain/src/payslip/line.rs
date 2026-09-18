use platform_kernel::Money;

use super::WorkMinutes;
use crate::project::ProjectId;

/// 給与明細の1行。どの案件で、どれだけ働き、時給はいくらだったか
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayslipLine {
    /// 稼働した案件
    project_id: ProjectId,
    /// その案件での、この月の稼働時間
    work_minutes: WorkMinutes,
    /// その案件での時給(円)。同じ派遣社員でも案件ごとに異なる
    hourly_rate: Money,
}

impl PayslipLine {
    #[must_use]
    pub fn new(project_id: ProjectId, work_minutes: WorkMinutes, hourly_rate: Money) -> Self {
        Self { project_id, work_minutes, hourly_rate }
    }

    #[must_use]
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }

    #[must_use]
    pub fn work_minutes(&self) -> WorkMinutes {
        self.work_minutes
    }

    #[must_use]
    pub fn hourly_rate(&self) -> Money {
        self.hourly_rate
    }

    /// この行の支給額。時給 × 稼働分 ÷ 60 で、円未満は切り捨てる
    /// (1,001円で15分なら 250.25円 → 250円)
    #[must_use]
    pub fn amount(&self) -> Money {
        (self.hourly_rate * self.work_minutes.as_minutes()).div_floor(60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amount_truncates_below_one_yen() {
        let line = PayslipLine::new(
            ProjectId::from_i64(1).unwrap(),
            WorkMinutes::from_minutes(15).unwrap(),
            Money::from_yen(1_001).unwrap(),
        );
        // 1,001円 × 15分 ÷ 60 = 250.25円 → 250円
        assert_eq!(line.amount().as_yen(), 250);
    }
}
