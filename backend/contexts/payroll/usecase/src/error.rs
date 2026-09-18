use payroll_domain::payslip::PayslipError;
use payroll_domain::project::ProjectError;
use payroll_domain::repository::RepositoryError;
use payroll_domain::staff::StaffError;
use platform_kernel::MoneyError;
use thiserror::Error;

use crate::ports::payout_gateway::PayoutError;
use crate::ports::user_directory::UserDirectoryError;

/// usecase が返すエラー。gRPC のステータスへの翻訳は handler が行う
#[derive(Debug, Error)]
pub enum UseCaseError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("見つかりません")]
    NotFound,
    #[error("既に存在します: {0}")]
    Conflict(String),
    #[error("{0}")]
    FailedPrecondition(String),
    #[error("依存先が利用できません: {0}")]
    Unavailable(String),
    #[error("内部エラー: {0}")]
    Internal(String),
}

impl From<PayslipError> for UseCaseError {
    fn from(err: PayslipError) -> Self {
        match err {
            PayslipError::AlreadyFinalized => Self::FailedPrecondition(err.to_string()),
            _ => Self::InvalidInput(err.to_string()),
        }
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

impl From<MoneyError> for UseCaseError {
    fn from(err: MoneyError) -> Self {
        Self::InvalidInput(err.to_string())
    }
}

impl From<RepositoryError> for UseCaseError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::Conflict(msg) => Self::Conflict(msg),
            RepositoryError::CorruptedData(msg) => Self::Internal(msg),
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
            UserDirectoryError::Invalid(msg) => Self::InvalidInput(msg),
            UserDirectoryError::Unavailable(msg) => Self::Unavailable(msg),
        }
    }
}
