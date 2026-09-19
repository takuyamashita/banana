use timesheet_usecase::UseCaseError;
use tonic::Status;

// domain・usecaseのエラーをgRPCのコードに翻訳するのもhandlerの責務
#[allow(clippy::needless_pass_by_value, reason = "map_err(to_status) で渡すため値で受ける")]
pub(crate) fn to_status(err: UseCaseError) -> Status {
    match &err {
        UseCaseError::InvalidInput(msg) => Status::invalid_argument(msg),
        UseCaseError::NotFound => Status::not_found(err.to_string()),
        UseCaseError::Conflict(msg) => Status::already_exists(msg),
        UseCaseError::FailedPrecondition(msg) => Status::failed_precondition(msg),
        UseCaseError::Unavailable(_) => {
            tracing::warn!(error = %err, "dependency unavailable");
            Status::unavailable("一時的に利用できません")
        }
        UseCaseError::Internal(_) => {
            // 内部の詳細はクライアントに返さずログにだけ残す
            tracing::error!(error = %err, "internal error");
            Status::internal("内部エラー")
        }
    }
}

pub(crate) fn invalid_argument(err: impl std::fmt::Display) -> Status {
    Status::invalid_argument(err.to_string())
}
