//! 勤怠で起きた、他の業務が知るべき出来事の記録

use async_trait::async_trait;
use timesheet_domain::timesheet::TimesheetEvent;

use super::database::Db;
use super::repository::RepositoryError;

/// 出来事の記録先。記録した出来事は、後から他の業務(給与)に届けられる
#[async_trait]
pub trait EventOutbox: Send + Sync {
    /// 出来事を記録する。記録は、同じトランザクションの他の記録と一緒に確定する
    async fn append(&self, db: &mut Db, event: TimesheetEvent) -> Result<(), RepositoryError>;
}
