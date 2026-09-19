//! 勤務表・派遣社員・案件の記録と取り出し。
//!
//! 勤務表は、新しく登録する `insert` と、登録済みの今の内容を記録する `update` を持つ。
//! 派遣社員と案件は給与(payroll)から届いた写しなので、届いた内容をそのまま記録する `save` だけを持つ
//! (同じものが2回届いても、記録は変わらない)

use async_trait::async_trait;
use platform_kernel::UserId;
use thiserror::Error;
use timesheet_domain::project::{Project, ProjectId};
use timesheet_domain::staff::{Staff, StaffId};
use timesheet_domain::timesheet::{NewTimesheet, Timesheet, TimesheetId, WorkMonth};

use super::database::Db;

/// 記録・取り出しができなかった理由
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// 既にある記録と重なる(同じ派遣社員・同じ月の勤務表など)
    #[error("既存のデータと衝突しました: {0}")]
    Conflict(String),
    /// 記録されている内容が業務ルールに合わず、取り出せない
    #[error("保存されたデータが壊れています: {0}")]
    CorruptedData(String),
    /// 記録の読み書きが一時的にできない。時間をおけば成功しうる
    #[error("リポジトリの操作に失敗しました: {0}")]
    Unavailable(String),
    /// やり直しても直らない異常(記録の形が想定と違う、書き込み先の用意の誤りなど)
    #[error("リポジトリの内部エラー: {0}")]
    Internal(String),
}

/// 勤務表の記録と取り出し
#[async_trait]
pub trait TimesheetRepository: Send + Sync {
    /// 勤務表番号で勤務表を探す
    async fn find(&self, id: TimesheetId) -> Result<Option<Timesheet>, RepositoryError>;
    /// 派遣社員のその月の勤務表を探す
    async fn find_by_staff_month(
        &self,
        staff_id: StaffId,
        month: WorkMonth,
    ) -> Result<Option<Timesheet>, RepositoryError>;
    /// 申告済み(承認を待っている)の勤務表を、申告した順に返す
    async fn list_submitted(&self) -> Result<Vec<Timesheet>, RepositoryError>;
    /// 勤務表番号で勤務表を探し、同じトランザクションが終わるまで他から変更されないようにする。
    /// 書き込み先はトランザクションでなければならない。読んだ内容を確かめてから書き戻すときに使う
    async fn find_for_update(
        &self,
        db: &mut Db,
        id: TimesheetId,
    ) -> Result<Option<Timesheet>, RepositoryError>;
    /// 新しい勤務表を登録し、振られた勤務表番号を返す。同じ派遣社員・同じ月の勤務表があれば `Conflict`
    async fn insert(&self, db: &mut Db, new: &NewTimesheet)
    -> Result<TimesheetId, RepositoryError>;
    /// 登録済みの勤務表の今の内容(状態・稼働の行を含む)を記録する。記録されていなければ `Internal`
    async fn update(&self, db: &mut Db, timesheet: &Timesheet) -> Result<(), RepositoryError>;
}

/// 派遣社員(給与から届いた写し)の記録と取り出し
#[async_trait]
pub trait StaffRepository: Send + Sync {
    async fn find(&self, id: StaffId) -> Result<Option<Staff>, RepositoryError>;
    /// ログイン用アカウントで派遣社員を探す。ログインした本人がどの派遣社員かを知るのに使う
    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Option<Staff>, RepositoryError>;
    /// 届いた内容を記録する。同じ派遣社員番号の記録があれば、届いた内容に置き換える
    async fn save(&self, db: &mut Db, staff: &Staff) -> Result<(), RepositoryError>;
}

/// 案件(給与から届いた写し)の記録と取り出し
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn find(&self, id: ProjectId) -> Result<Option<Project>, RepositoryError>;
    /// 案件を案件番号の順に返す
    async fn list(&self) -> Result<Vec<Project>, RepositoryError>;
    /// 届いた内容を記録する。同じ案件番号の記録があれば、届いた内容に置き換える
    async fn save(&self, db: &mut Db, project: &Project) -> Result<(), RepositoryError>;
}
