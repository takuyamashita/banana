//! 案件・派遣社員・給与明細・振込依頼の記録と取り出し。
//!
//! 記録(`insert`・`update`)は書き込み先([`Db`])を受け取る。同じトランザクションに書いた記録は、
//! まとめて確定するか、まとめて取り消される。
//!
//! [`Db`]: super::database::Db

use async_trait::async_trait;
use payroll_domain::payout::{NewPayout, Payout, PayoutId};
use payroll_domain::payslip::{NewPayslip, Payslip, PayslipId};
use payroll_domain::project::{NewProject, Project, ProjectId};
use payroll_domain::staff::{NewStaff, Staff, StaffId};
use platform_kernel::{Email, UserId};
use thiserror::Error;

use super::database::Db;

/// 記録・取り出しができなかった理由
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// 既にある記録と重なる(同じメールアドレスの派遣社員、同じ月の給与明細など)
    #[error("既存のデータと衝突しました: {0}")]
    Conflict(String),
    /// 記録されている内容が業務ルールに合わず、取り出せない
    #[error("保存されたデータが壊れています: {0}")]
    CorruptedData(String),
    /// 記録の読み書きが一時的にできない
    #[error("リポジトリの操作に失敗しました: {0}")]
    Unavailable(String),
}

/// 給与明細の記録と取り出し
#[async_trait]
pub trait PayslipRepository: Send + Sync {
    /// 給与明細番号で給与明細を探す
    async fn find(&self, id: PayslipId) -> Result<Option<Payslip>, RepositoryError>;
    /// 派遣社員の有効な給与明細を、新しい月から順に返す
    async fn list_by_staff(&self, staff_id: StaffId) -> Result<Vec<Payslip>, RepositoryError>;
    /// 給与明細番号で給与明細を探し、同じトランザクションが終わるまで他から変更されないようにする。
    /// 読んだ内容を確かめてから書き戻すとき(確定など)に使う
    async fn find_for_update(
        &self,
        db: &mut Db,
        id: PayslipId,
    ) -> Result<Option<Payslip>, RepositoryError>;
    /// 新しい給与明細を登録し、振られた給与明細番号を返す
    async fn insert(&self, db: &mut Db, new: &NewPayslip) -> Result<PayslipId, RepositoryError>;
    /// 登録済みの給与明細の変更(状態の変化など)を記録する
    async fn update(&self, db: &mut Db, payslip: &Payslip) -> Result<(), RepositoryError>;
}

/// 派遣社員の記録と取り出し
#[async_trait]
pub trait StaffRepository: Send + Sync {
    /// 派遣社員番号で派遣社員を探す
    async fn find(&self, id: StaffId) -> Result<Option<Staff>, RepositoryError>;
    /// ログイン用アカウントから、その持ち主の派遣社員を探す
    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Option<Staff>, RepositoryError>;
    /// メールアドレスで派遣社員を探す
    async fn find_by_email(&self, email: &Email) -> Result<Option<Staff>, RepositoryError>;
    /// 新しい派遣社員を登録し、振られた派遣社員番号を返す
    async fn insert(&self, db: &mut Db, new: &NewStaff) -> Result<StaffId, RepositoryError>;
}

/// 案件の記録と取り出し
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// 案件番号で案件を探す
    async fn find(&self, id: ProjectId) -> Result<Option<Project>, RepositoryError>;
    /// 新しい案件を登録し、振られた案件番号を返す
    async fn insert(&self, db: &mut Db, new: &NewProject) -> Result<ProjectId, RepositoryError>;
}

/// 振込依頼の記録と取り出し
#[async_trait]
pub trait PayoutRepository: Send + Sync {
    /// 給与明細の振込依頼を探す。1つの給与明細につき振込依頼は1つ
    async fn find_by_payslip(
        &self,
        payslip_id: PayslipId,
    ) -> Result<Option<Payout>, RepositoryError>;
    /// 振込依頼を記録し、振られた振込依頼番号を返す
    async fn insert(&self, db: &mut Db, new: &NewPayout) -> Result<PayoutId, RepositoryError>;
}
