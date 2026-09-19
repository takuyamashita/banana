use platform_kernel::Money;

use super::{HourlyRate, PayslipError, WorkMinutes};
use crate::project::ProjectId;

/// 給与明細の1行。どの案件で、どれだけ働き、時給はいくらだったか
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayslipLine {
    /// 稼働した案件
    project_id: ProjectId,
    /// その案件での、この月の稼働時間
    work_minutes: WorkMinutes,
    /// その案件での時給
    hourly_rate: HourlyRate,
    /// この行の支給額。時給 × 稼働分 ÷ 60 で、円未満は切り捨てる
    amount: Money,
}

impl PayslipLine {
    /// 明細行を作り、支給額を決める(1,001円で15分なら 250.25円 → 250円)
    pub fn new(
        project_id: ProjectId,
        work_minutes: WorkMinutes,
        hourly_rate: HourlyRate,
    ) -> Result<Self, PayslipError> {
        let yen = i64::from(hourly_rate.as_yen()) * i64::from(work_minutes.as_minutes()) / 60;
        let amount = Money::from_yen(yen)
            .map_err(|_| PayslipError::InvalidHourlyRate { max: HourlyRate::MAX_YEN })?;
        Ok(Self { project_id, work_minutes, hourly_rate, amount })
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
    pub fn hourly_rate(&self) -> HourlyRate {
        self.hourly_rate
    }

    /// この行の支給額(円未満は切り捨て済み)
    #[must_use]
    pub fn amount(&self) -> Money {
        self.amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(minutes: u32, rate: i64) -> PayslipLine {
        PayslipLine::new(
            ProjectId::from_i64(1).unwrap(),
            WorkMinutes::from_minutes(minutes).unwrap(),
            HourlyRate::from_yen(rate).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn amount_truncates_below_one_yen() {
        // 1,001円 × 15分 ÷ 60 = 250.25円 → 250円
        assert_eq!(line(15, 1_001).amount().as_yen(), 250);
    }

    #[test]
    fn the_largest_line_fits() {
        // 10万円 × 744時間 = 7,440万円
        assert_eq!(line(744 * 60, 100_000).amount().as_yen(), 74_400_000);
    }
}
