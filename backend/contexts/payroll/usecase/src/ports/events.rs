//! 給与計算で起きた、他の業務が知るべき出来事の記録

use async_trait::async_trait;
use payroll_domain::payslip::PayslipEvent;
use payroll_domain::project::ProjectEvent;
use payroll_domain::staff::StaffEvent;

use super::database::Db;
use super::repository::RepositoryError;

/// 給与計算で起きた出来事
#[must_use = "出来事は記録して後続の業務に知らせる必要がある"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayrollEvent {
    /// 給与明細に起きた出来事
    Payslip(PayslipEvent),
    /// 派遣社員に起きた出来事
    Staff(StaffEvent),
    /// 案件に起きた出来事
    Project(ProjectEvent),
}

/// 出来事の記録先。記録した出来事は、後から後続の業務(振込・勤怠など)に届けられる
#[async_trait]
pub trait EventOutbox: Send + Sync {
    /// 出来事を記録する。記録は、同じトランザクションの他の記録と一緒に確定する
    async fn append(&self, db: &mut Db, event: PayrollEvent) -> Result<(), RepositoryError>;
}
