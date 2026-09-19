use platform_kernel::InvalidId;
use thiserror::Error;
use timesheet_domain::project::ProjectError;
use timesheet_domain::staff::StaffError;
use timesheet_domain::timesheet::TimesheetError;

use crate::ports::repository::RepositoryError;

/// 操作が成り立たなかった理由
#[derive(Debug, Error)]
pub enum UseCaseError {
    /// 入力が業務ルールに合わない(15分単位でない稼働、対象月の外の日、登録されていない案件など)
    #[error("{0}")]
    InvalidInput(String),
    /// 対象が見つからない。見る権限のないものも、見つからないものとして扱う
    #[error("見つかりません")]
    NotFound,
    /// 既にあるものと重なる
    #[error("既に存在します: {0}")]
    Conflict(String),
    /// 今の状態ではできない(申告した勤務表を書き直す、申告していない勤務表を承認するなど)
    #[error("{0}")]
    FailedPrecondition(String),
    /// 記録の読み書きが一時的にできない。時間をおけば成功しうる
    #[error("依存先が利用できません: {0}")]
    Unavailable(String),
    /// 記録されている内容が業務ルールに合わないなど、利用者には対処できない異常
    #[error("内部エラー: {0}")]
    Internal(String),
}

impl From<TimesheetError> for UseCaseError {
    fn from(err: TimesheetError) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

impl From<StaffError> for UseCaseError {
    fn from(err: StaffError) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

impl From<ProjectError> for UseCaseError {
    fn from(err: ProjectError) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

impl From<InvalidId> for UseCaseError {
    fn from(err: InvalidId) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

impl From<RepositoryError> for UseCaseError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::Conflict(msg) => Self::Conflict(msg),
            RepositoryError::CorruptedData(msg) | RepositoryError::Internal(msg) => {
                Self::Internal(msg)
            }
            RepositoryError::Unavailable(msg) => Self::Unavailable(msg),
        }
    }
}
