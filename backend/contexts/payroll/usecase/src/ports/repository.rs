//! 集約の出し入れ口。domain は永続化を知らないので、trait はここ(usecase)に置く

use async_trait::async_trait;
use payroll_domain::payslip::{NewPayslip, Payslip, PayslipId};
use payroll_domain::project::{NewProject, ProjectId};
use payroll_domain::staff::{Email, NewStaff, Staff, StaffId, UserId};
use thiserror::Error;

/// リポジトリ実装が返すエラー。sqlx などの下位エラーは infrastructure でこれに翻訳する
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// 一意制約違反など、既存データと衝突した
    #[error("既存のデータと衝突しました: {0}")]
    Conflict(String),
    /// DB に想定外の値が入っていて、集約を組み立てられない
    #[error("保存されたデータが壊れています: {0}")]
    CorruptedData(String),
    /// 接続断など、呼び出し側では対処できない失敗
    #[error("リポジトリの操作に失敗しました: {0}")]
    Unavailable(String),
}

// 採番があるため insert と update を分ける。insert は採番済みの Payslip を返す
#[async_trait]
pub trait PayslipRepository: Send + Sync {
    async fn insert(&self, new: &mut NewPayslip) -> Result<Payslip, RepositoryError>;
    async fn update(&self, payslip: &mut Payslip) -> Result<(), RepositoryError>;
    async fn find(&self, id: PayslipId) -> Result<Option<Payslip>, RepositoryError>;
    /// 派遣社員の有効な給与明細。合計は業務ルール(円未満切り捨て)で決まるので、
    /// 一覧でも集約として返す(合計を保存するようになったら queries に移せる)
    async fn list_by_staff(&self, staff_id: StaffId) -> Result<Vec<Payslip>, RepositoryError>;
}

#[async_trait]
pub trait StaffRepository: Send + Sync {
    async fn insert(&self, new: &NewStaff) -> Result<StaffId, RepositoryError>;
    async fn find(&self, id: StaffId) -> Result<Option<Staff>, RepositoryError>;
    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Option<Staff>, RepositoryError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<Staff>, RepositoryError>;
}

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn insert(&self, new: &NewProject) -> Result<ProjectId, RepositoryError>;
}
