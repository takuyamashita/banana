use std::sync::Arc;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use platform_auth::{AuthError, OidcVerifier};
use platform_kernel::{AuthenticatedUser, Role};
use tonic::Status;

/// 認証ミドルウェア。Bearer トークンを検証できたら `AuthenticatedUser` を extensions に載せる。
/// トークンがない・不正なときは載せずに通し、ハンドラ側で Unauthenticated を返す
pub async fn authenticate(
    State(verifier): State<Arc<OidcVerifier>>,
    mut request: Request,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if let Some(token) = token {
        match verifier.verify(token).await {
            Ok(user) => {
                request.extensions_mut().insert(user);
            }
            // 認証基盤に届かないのはトークンの問題ではない。再ログインを促さず、時間をおいてもらう
            Err(AuthError::KeyUnavailable(err)) => {
                tracing::warn!(error = %err, "cannot verify tokens: signing keys unavailable");
                return Status::unavailable("一時的に利用できません").into_http();
            }
            Err(err) => tracing::info!(error = %err, "token rejected"),
        }
    }
    next.run(request).await
}

pub(crate) fn current_user<T>(request: &tonic::Request<T>) -> Result<AuthenticatedUser, Status> {
    request
        .extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or_else(|| Status::unauthenticated("ログインしてください"))
}

pub(crate) fn require_admin<T>(request: &tonic::Request<T>) -> Result<AuthenticatedUser, Status> {
    let user = current_user(request)?;
    if !user.has_role(Role::Admin) {
        return Err(Status::permission_denied("この操作は管理者だけができます"));
    }
    Ok(user)
}
