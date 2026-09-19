use payroll_domain::payslip::PayslipError;
use payroll_domain::project::ProjectError;
use payroll_domain::staff::StaffError;
use payroll_domain::work::WorkError;
use platform_kernel::MoneyError;
use thiserror::Error;

use crate::ports::payout_gateway::PayoutError;
use crate::ports::repository::RepositoryError;
use crate::ports::user_directory::UserDirectoryError;

/// 操作が成り立たなかった理由
#[derive(Debug, Error)]
pub enum UseCaseError {
    /// 入力が業務ルールに合わない(存在しない派遣社員、範囲外の月など)
    #[error("{0}")]
    InvalidInput(String),
    /// 対象が見つからない。見る権限のないものも、見つからないものとして扱う
    #[error("見つかりません")]
    NotFound,
    /// 既にあるものと重なる(同じメールアドレスの派遣社員、同じ月の給与明細など)
    #[error("既に存在します: {0}")]
    Conflict(String),
    /// 今の状態ではできない(確定済みの給与明細をもう一度確定する、振込を断られたなど)
    #[error("{0}")]
    FailedPrecondition(String),
    /// 記録の読み書きや外部への依頼が一時的にできない。時間をおけば成功しうる
    #[error("依存先が利用できません: {0}")]
    Unavailable(String),
    /// 記録されている内容が業務ルールに合わないなど、利用者には対処できない異常
    #[error("内部エラー: {0}")]
    Internal(String),
}

impl From<PayslipError> for UseCaseError {
    fn from(err: PayslipError) -> Self {
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

impl From<WorkError> for UseCaseError {
    fn from(err: WorkError) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

impl From<MoneyError> for UseCaseError {
    fn from(err: MoneyError) -> Self {
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

impl From<PayoutError> for UseCaseError {
    fn from(err: PayoutError) -> Self {
        match err {
            PayoutError::Rejected(msg) => Self::FailedPrecondition(msg),
            PayoutError::Unavailable(msg) => Self::Unavailable(msg),
        }
    }
}

impl From<UserDirectoryError> for UseCaseError {
    fn from(err: UserDirectoryError) -> Self {
        match err {
            UserDirectoryError::AlreadyExists => Self::Conflict(err.to_string()),
            UserDirectoryError::InvalidPassword | UserDirectoryError::Invalid { .. } => {
                Self::InvalidInput(err.to_string())
            }
            UserDirectoryError::Unavailable(msg) => Self::Unavailable(msg),
        }
    }
}
