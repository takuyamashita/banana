use std::time::{Duration, Instant};

use async_trait::async_trait;
use payroll_usecase::ports::user_directory::{UserDirectory, UserDirectoryError};
use platform_kernel::{Email, UserId};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::sync::Mutex;

/// ローカル用。Keycloak の Admin REST API を使う。
///
/// 管理用トークンは同じ realm のサービスアカウント付きクライアント(`client_credentials`)で取る。
/// master realm のクライアントは realm import で作れないため
pub struct KeycloakUserDirectory {
    client: reqwest::Client,
    base_url: String, // http://localhost:8080
    realm: String,    // platform
    client_id: String,
    client_secret: String,
    token: Mutex<Option<(String, Instant)>>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

fn unavailable(err: impl std::fmt::Display) -> UserDirectoryError {
    UserDirectoryError::Unavailable(err.to_string())
}

impl KeycloakUserDirectory {
    #[must_use]
    pub fn new(
        client: reqwest::Client,
        base_url: String,
        realm: String,
        client_id: String,
        client_secret: String,
    ) -> Self {
        Self { client, base_url, realm, client_id, client_secret, token: Mutex::new(None) }
    }

    /// 管理用トークンを期限の30秒前までキャッシュする
    async fn admin_token(&self) -> Result<String, UserDirectoryError> {
        let mut cached = self.token.lock().await;
        if let Some((token, expires_at)) = cached.as_ref()
            && Instant::now() < *expires_at
        {
            return Ok(token.clone());
        }

        let res: TokenResponse = self
            .client
            .post(format!("{}/realms/{}/protocol/openid-connect/token", self.base_url, self.realm))
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await
            .map_err(unavailable)?
            .error_for_status()
            .map_err(unavailable)?
            .json()
            .await
            .map_err(unavailable)?;

        let ttl = Duration::from_secs(res.expires_in.saturating_sub(30));
        *cached = Some((res.access_token.clone(), Instant::now() + ttl));
        Ok(res.access_token)
    }

    fn users_url(&self) -> String {
        format!("{}/admin/realms/{}/users", self.base_url, self.realm)
    }

    /// realm ロール staff を付ける。ロールの表現(id と name)を取ってから割り当てる
    async fn grant_staff_role(&self, id: &UserId) -> Result<(), UserDirectoryError> {
        let token = self.admin_token().await?;
        let role: serde_json::Value = self
            .client
            .get(format!("{}/admin/realms/{}/roles/staff", self.base_url, self.realm))
            .bearer_auth(&token)
            .send()
            .await
            .map_err(unavailable)?
            .error_for_status()
            .map_err(unavailable)?
            .json()
            .await
            .map_err(unavailable)?;
        self.client
            .post(format!("{}/{}/role-mappings/realm", self.users_url(), id.as_str()))
            .bearer_auth(&token)
            .json(&[role])
            .send()
            .await
            .map_err(unavailable)?
            .error_for_status()
            .map_err(unavailable)?;
        Ok(())
    }
}

#[async_trait]
impl UserDirectory for KeycloakUserDirectory {
    async fn create_user(
        &self,
        email: &Email,
        temporary_password: &str,
    ) -> Result<UserId, UserDirectoryError> {
        // Admin REST API: POST /admin/realms/{realm}/users
        let res = self
            .client
            .post(self.users_url())
            .bearer_auth(self.admin_token().await?)
            .json(&serde_json::json!({
                "username": email.as_str(),
                "email": email.as_str(),
                "emailVerified": true,
                "enabled": true,
                "credentials": [{ "type": "password", "value": temporary_password, "temporary": true }],
            }))
            .send()
            .await
            .map_err(unavailable)?;

        match res.status() {
            StatusCode::CREATED => {}
            StatusCode::CONFLICT => return Err(UserDirectoryError::AlreadyExists),
            StatusCode::BAD_REQUEST => {
                let body = res.text().await.unwrap_or_default();
                // パスワードポリシーに合わないときは errorMessage が invalidPassword… になる
                if body.contains("invalidPassword") {
                    return Err(UserDirectoryError::InvalidPassword);
                }
                tracing::info!(detail = body, "keycloak rejected the new user");
                return Err(UserDirectoryError::Invalid { detail: body });
            }
            other => return Err(UserDirectoryError::Unavailable(other.to_string())),
        }

        // KeycloakはLocationヘッダに作成したIDを返す。この差もここに閉じる
        let id = res
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|loc| loc.rsplit('/').next())
            .ok_or_else(|| UserDirectoryError::Unavailable("Location header missing".into()))?;
        let user_id =
            UserId::parse(id).map_err(|e| UserDirectoryError::Unavailable(e.to_string()))?;

        // 派遣社員のロール(realm ロール staff)を付ける
        if let Err(err) = self.grant_staff_role(&user_id).await {
            let _ = self.delete_user(&user_id).await;
            return Err(err);
        }
        Ok(user_id)
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), UserDirectoryError> {
        // DELETE /admin/realms/{realm}/users/{id}
        let res = self
            .client
            .delete(format!("{}/{}", self.users_url(), id.as_str()))
            .bearer_auth(self.admin_token().await?)
            .send()
            .await
            .map_err(unavailable)?;
        // 既にないなら、消したのと同じ
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(());
        }
        res.error_for_status().map_err(unavailable)?;
        Ok(())
    }
}
