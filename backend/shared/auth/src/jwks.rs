use std::collections::HashMap;
use std::time::{Duration, Instant};

use jsonwebtoken::DecodingKey;
use jsonwebtoken::jwk::JwkSet;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::AuthError;

#[derive(Deserialize)]
struct Discovery {
    jwks_uri: String,
}

struct State {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Option<Instant>,
}

/// kid → 署名鍵のキャッシュ。未知の kid で取り直すが、`min_refresh` より短い間隔では取りに行かない
pub(crate) struct JwksCache {
    client: reqwest::Client,
    issuer: String,
    min_refresh: Duration,
    state: RwLock<State>,
}

fn unavailable(err: impl std::fmt::Display) -> AuthError {
    AuthError::KeyUnavailable(err.to_string())
}

impl JwksCache {
    pub(crate) fn new(client: reqwest::Client, issuer: String, min_refresh: Duration) -> Self {
        Self {
            client,
            issuer,
            min_refresh,
            state: RwLock::new(State { keys: HashMap::new(), fetched_at: None }),
        }
    }

    pub(crate) async fn key(&self, kid: &str) -> Result<DecodingKey, AuthError> {
        if let Some(key) = self.state.read().await.keys.get(kid) {
            return Ok(key.clone());
        }

        let mut state = self.state.write().await;
        // 待っている間に別のリクエストが取り直しているかもしれない
        if let Some(key) = state.keys.get(kid) {
            return Ok(key.clone());
        }
        if state.fetched_at.is_some_and(|t| t.elapsed() < self.min_refresh) {
            return Err(AuthError::InvalidToken(format!("unknown kid: {kid}")));
        }

        state.keys = self.fetch().await?;
        state.fetched_at = Some(Instant::now());
        tracing::info!(keys = state.keys.len(), "JWKS refreshed");

        state
            .keys
            .get(kid)
            .cloned()
            .ok_or_else(|| AuthError::InvalidToken(format!("unknown kid: {kid}")))
    }

    async fn fetch(&self) -> Result<HashMap<String, DecodingKey>, AuthError> {
        let discovery: Discovery = self
            .client
            .get(format!("{}/.well-known/openid-configuration", self.issuer))
            .send()
            .await
            .map_err(unavailable)?
            .error_for_status()
            .map_err(unavailable)?
            .json()
            .await
            .map_err(unavailable)?;

        let set: JwkSet = self
            .client
            .get(discovery.jwks_uri)
            .send()
            .await
            .map_err(unavailable)?
            .error_for_status()
            .map_err(unavailable)?
            .json()
            .await
            .map_err(unavailable)?;

        Ok(set
            .keys
            .iter()
            .filter_map(|jwk| {
                let kid = jwk.common.key_id.clone()?;
                DecodingKey::from_jwk(jwk).ok().map(|key| (kid, key))
            })
            .collect())
    }
}
