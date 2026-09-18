use jsonwebtoken::Validation;
use platform_kernel::{AuthenticatedUser, Role, UserId};
use serde_json::Value;

use crate::AuthError;

/// プロバイダごとのクレームの差をここに閉じ込める
#[derive(Debug, Clone)]
pub enum ClaimMapper {
    /// ロールは `realm_access.roles`。aud は既定では `account` なので、
    /// クライアントに audience mapper を付けて API 識別子を入れ、それを検証する
    Keycloak { audience: String },
    /// ロールは `cognito:groups`。アクセストークンに aud はなく、`client_id` と `token_use` を見る
    Cognito { client_id: String },
}

impl ClaimMapper {
    pub(crate) fn configure(&self, validation: &mut Validation) {
        match self {
            Self::Keycloak { audience } => validation.set_audience(&[audience]),
            Self::Cognito { .. } => validation.validate_aud = false,
        }
    }

    pub(crate) fn map(&self, claims: &Value) -> Result<AuthenticatedUser, AuthError> {
        let user_id = claims
            .get("sub")
            .and_then(Value::as_str)
            .ok_or_else(|| AuthError::InvalidToken("sub missing".into()))
            .and_then(|sub| {
                UserId::parse(sub).map_err(|e| AuthError::InvalidToken(e.to_string()))
            })?;

        let role_names = match self {
            Self::Keycloak { .. } => claims.pointer("/realm_access/roles"),
            Self::Cognito { client_id } => {
                if claims.get("token_use").and_then(Value::as_str) != Some("access") {
                    return Err(AuthError::InvalidToken("not an access token".into()));
                }
                if claims.get("client_id").and_then(Value::as_str) != Some(client_id) {
                    return Err(AuthError::InvalidToken("client_id mismatch".into()));
                }
                claims.get("cognito:groups")
            }
        };

        let roles = role_names
            .and_then(Value::as_array)
            .map(|names| {
                names.iter().filter_map(Value::as_str).filter_map(Role::from_name).collect()
            })
            .unwrap_or_default();

        Ok(AuthenticatedUser { user_id, roles })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keycloak_roles_come_from_realm_access() {
        let mapper = ClaimMapper::Keycloak { audience: "platform-api".into() };
        let user = mapper
            .map(&json!({ "sub": "u1", "realm_access": { "roles": ["admin", "offline_access"] } }))
            .unwrap();
        assert_eq!(user.roles, vec![Role::Admin]);
    }

    #[test]
    fn cognito_roles_come_from_groups() {
        let mapper = ClaimMapper::Cognito { client_id: "c1".into() };
        let user = mapper
            .map(&json!({ "sub": "u1", "token_use": "access", "client_id": "c1", "cognito:groups": ["staff"] }))
            .unwrap();
        assert_eq!(user.roles, vec![Role::Staff]);
    }

    #[test]
    fn cognito_id_token_is_rejected() {
        let mapper = ClaimMapper::Cognito { client_id: "c1".into() };
        assert!(mapper.map(&json!({ "sub": "u1", "token_use": "id", "client_id": "c1" })).is_err());
    }

    #[test]
    fn cognito_other_client_is_rejected() {
        let mapper = ClaimMapper::Cognito { client_id: "c1".into() };
        assert!(
            mapper.map(&json!({ "sub": "u1", "token_use": "access", "client_id": "c2" })).is_err()
        );
    }
}
