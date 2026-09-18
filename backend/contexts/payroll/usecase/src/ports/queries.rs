//! 一覧画面に出すための情報の取り出し

use async_trait::async_trait;
use payroll_domain::project::ProjectId;
use payroll_domain::staff::StaffId;

use super::repository::RepositoryError;

/// 案件一覧の1行
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectView {
    /// 案件番号
    pub id: ProjectId,
    /// 案件名
    pub name: String,
}

/// 派遣社員一覧の1行
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffView {
    /// 派遣社員番号
    pub id: StaffId,
    /// メールアドレス
    pub email: String,
    /// 表示名
    pub display_name: String,
}

#[async_trait]
pub trait ProjectQuery: Send + Sync {
    /// 登録済みの全案件を、登録した順に返す
    async fn list(&self) -> Result<Vec<ProjectView>, RepositoryError>;
}

#[async_trait]
pub trait StaffQuery: Send + Sync {
    /// 登録済みの全派遣社員を、登録した順に返す
    async fn list(&self) -> Result<Vec<StaffView>, RepositoryError>;
}
