use std::collections::HashMap;
use std::time::{Duration, Instant};

use jsonwebtoken::DecodingKey;
use jsonwebtoken::jwk::{Jwk, JwkSet, KeyAlgorithm, PublicKeyUse};
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};

use crate::AuthError;

#[derive(Deserialize)]
struct Discovery {
    jwks_uri: String,
}

struct State {
    keys: HashMap<String, DecodingKey>,
    /// 最後に取り直そうとした時刻(失敗も含む)
    last_attempt: Option<Instant>,
    /// 最後の取り直しが失敗したか
    last_failed: bool,
}

/// kid → 署名鍵のキャッシュ。未知の kid で取り直すが、`min_refresh` より短い間隔では取りに行かない。
///
/// 取り直しは1本にまとめ、その間もキャッシュ済みの kid の検証は止めない(読み取りのロックだけで済む)。
/// 失敗した取り直しも間隔に数えるので、認証基盤に届かないときに、でたらめな kid のトークンを
/// 送り続けられても取りに行き続けない
pub(crate) struct JwksCache {
    client: reqwest::Client,
    issuer: String,
    min_refresh: Duration,
    state: RwLock<State>,
    refreshing: Mutex<()>,
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
            state: RwLock::new(State {
                keys: HashMap::new(),
                last_attempt: None,
                last_failed: false,
            }),
            refreshing: Mutex::new(()),
        }
    }

    /// 取り直した直後として、鍵を持たせる(テスト用)
    #[cfg(test)]
    pub(crate) fn with_keys(
        client: reqwest::Client,
        issuer: String,
        min_refresh: Duration,
        keys: HashMap<String, DecodingKey>,
    ) -> Self {
        let cache = Self::new(client, issuer, min_refresh);
        *cache.state.try_write().expect("作ったばかりのロック") =
            State { keys, last_attempt: Some(Instant::now()), last_failed: false };
        cache
    }

    pub(crate) async fn key(&self, kid: &str) -> Result<DecodingKey, AuthError> {
        if let Some(key) = self.cached(kid).await {
            return Ok(key);
        }

        let _refreshing = self.refreshing.lock().await;
        // 待っている間に、別のリクエストが取り直しているかもしれない
        if let Some(key) = self.cached(kid).await {
            return Ok(key);
        }
        {
            let state = self.state.read().await;
            if state.last_attempt.is_some_and(|t| t.elapsed() < self.min_refresh) {
                return Err(if state.last_failed {
                    AuthError::KeyUnavailable("JWKS の取得に失敗したばかりです".into())
                } else {
                    AuthError::InvalidToken(format!("unknown kid: {kid}"))
                });
            }
        }

        // 取りに行く間は書き込みのロックを持たない(キャッシュ済みの kid の検証を止めない)
        let fetched = self.fetch().await;
        let mut state = self.state.write().await;
        state.last_attempt = Some(Instant::now());
        state.last_failed = fetched.is_err();
        state.keys = fetched?;
        tracing::info!(keys = state.keys.len(), "JWKS refreshed");
        state
            .keys
            .get(kid)
            .cloned()
            .ok_or_else(|| AuthError::InvalidToken(format!("unknown kid: {kid}")))
    }

    async fn cached(&self, kid: &str) -> Option<DecodingKey> {
        self.state.read().await.keys.get(kid).cloned()
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
            .filter(|jwk| is_rs256_signing_key(jwk))
            .filter_map(|jwk| {
                let kid = jwk.common.key_id.clone()?;
                DecodingKey::from_jwk(jwk).ok().map(|key| (kid, key))
            })
            .collect())
    }
}

/// 署名の検証に使う鍵だけを残す。Keycloak は暗号化用(use: enc、RSA-OAEP)の鍵も公開している
fn is_rs256_signing_key(jwk: &Jwk) -> bool {
    let for_signing =
        jwk.common.public_key_use.as_ref().is_none_or(|u| *u == PublicKeyUse::Signature);
    let rs256 = jwk.common.key_algorithm.is_none_or(|a| a == KeyAlgorithm::RS256);
    for_signing && rs256
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    /// 接続を受けたら数えてすぐ切る、応答しない認証基盤
    async fn broken_issuer() -> (String, Arc<AtomicUsize>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let accepted = Arc::new(AtomicUsize::new(0));
        let counter = accepted.clone();
        tokio::spawn(async move {
            while let Ok((socket, _)) = listener.accept().await {
                counter.fetch_add(1, Ordering::SeqCst);
                drop(socket);
            }
        });
        (format!("http://{addr}"), accepted)
    }

    #[tokio::test]
    async fn a_failed_refresh_is_not_retried_until_the_interval_passes() {
        let (issuer, accepted) = broken_issuer().await;
        let cache = JwksCache::new(reqwest::Client::new(), issuer, Duration::from_secs(30));

        // 1回目は取りに行って失敗する。認証基盤に届かないことはトークンの問題ではない
        assert!(matches!(cache.key("a").await, Err(AuthError::KeyUnavailable(_))));
        // でたらめな kid が続いても、間隔が空くまでは取りに行かない
        assert!(matches!(cache.key("b").await, Err(AuthError::KeyUnavailable(_))));
        assert!(matches!(cache.key("c").await, Err(AuthError::KeyUnavailable(_))));
        assert_eq!(accepted.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn only_rs256_signing_keys_are_used() {
        let jwk = |extra: &str| -> Jwk {
            serde_json::from_str(&format!(
                r#"{{"kty":"RSA","kid":"k","n":"AQAB","e":"AQAB"{extra}}}"#
            ))
            .unwrap()
        };
        assert!(is_rs256_signing_key(&jwk("")));
        assert!(is_rs256_signing_key(&jwk(r#","use":"sig","alg":"RS256""#)));
        assert!(!is_rs256_signing_key(&jwk(r#","use":"enc","alg":"RSA-OAEP""#)));
        assert!(!is_rs256_signing_key(&jwk(r#","use":"sig","alg":"RS512""#)));
    }
}
