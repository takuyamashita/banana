//! 画面向けの読み取り。集約を組み立てずに表示用の型を返す。
//! 並び順・ページング・表示項目の都合はここで吸収し、domain とリポジトリに持ち込まない

use async_trait::async_trait;
use payroll_domain::project::ProjectId;
use payroll_domain::staff::StaffId;

use super::repository::RepositoryError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectView {
    pub id: ProjectId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffView {
    pub id: StaffId,
    pub email: String,
    pub display_name: String,
}

#[async_trait]
pub trait ProjectQuery: Send + Sync {
    async fn list(&self) -> Result<Vec<ProjectView>, RepositoryError>;
}

#[async_trait]
pub trait StaffQuery: Send + Sync {
    async fn list(&self) -> Result<Vec<StaffView>, RepositoryError>;
}
