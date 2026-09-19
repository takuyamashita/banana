//! OIDC のアクセストークン検証。
//!
//! 署名鍵(JWKS)は kid ごとにキャッシュし、知らない kid が来たら取り直す(鍵ローテーションへの追従)。
//! プロバイダごとのクレームの差は [`ClaimMapper`] に閉じ込め、外には `AuthenticatedUser` だけを出す。

mod jwks;
mod mapper;
mod middleware;

use std::time::Duration;

use jsonwebtoken::{Algorithm, Validation, decode, decode_header};
use platform_kernel::AuthenticatedUser;
use thiserror::Error;

pub use mapper::ClaimMapper;
pub use middleware::{authenticate, current_user, require_admin};

use crate::jwks::JwksCache;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("トークンが不正です: {0}")]
    InvalidToken(String),
    #[error("署名鍵を取得できません: {0}")]
    KeyUnavailable(String),
}

pub struct OidcVerifier {
    issuer: String,
    mapper: ClaimMapper,
    jwks: JwksCache,
}

impl OidcVerifier {
    /// `issuer` は `iss` クレームと一致する URL。JWKS は `{issuer}/.well-known/openid-configuration` から辿る
    #[must_use]
    pub fn new(client: reqwest::Client, issuer: String, mapper: ClaimMapper) -> Self {
        let jwks = JwksCache::new(client, issuer.clone(), Duration::from_secs(30));
        Self { issuer, mapper, jwks }
    }

    /// 鍵を渡して作る(テスト用。JWKS を取りに行かない)
    #[cfg(test)]
    fn with_keys(
        issuer: &str,
        mapper: ClaimMapper,
        keys: std::collections::HashMap<String, jsonwebtoken::DecodingKey>,
    ) -> Self {
        let jwks = JwksCache::with_keys(
            reqwest::Client::new(),
            issuer.to_owned(),
            Duration::from_secs(30),
            keys,
        );
        Self { issuer: issuer.to_owned(), mapper, jwks }
    }

    pub async fn verify(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        let header = decode_header(token).map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        let kid = header.kid.ok_or_else(|| AuthError::InvalidToken("kid missing".into()))?;
        let key = self.jwks.key(&kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.validate_nbf = true;
        self.mapper.configure(&mut validation);

        let data = decode::<serde_json::Value>(token, &key, &validation)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        self.mapper.map(&data.claims)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    use jsonwebtoken::{DecodingKey, EncodingKey, Header, encode};
    use platform_kernel::Role;
    use serde_json::{Value, json};

    use super::*;

    const ISSUER: &str = "http://localhost:8080/realms/platform";
    const KID: &str = "test-kid";

    fn now() -> i64 {
        i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()).unwrap()
    }

    fn keycloak() -> OidcVerifier {
        let key =
            DecodingKey::from_rsa_pem(include_bytes!("../tests/fixtures/test-signing-key.pub.pem"))
                .unwrap();
        OidcVerifier::with_keys(
            ISSUER,
            ClaimMapper::Keycloak { audience: "platform-api".into() },
            HashMap::from([(KID.to_owned(), key)]),
        )
    }

    /// 正しいアクセストークンのクレーム。テストごとに一部を変える
    fn claims() -> Value {
        json!({
            "iss": ISSUER,
            "aud": "platform-api",
            "sub": "user-1",
            "typ": "Bearer",
            "exp": now() + 300,
            "realm_access": { "roles": ["admin"] },
        })
    }

    fn sign(claims: &Value) -> String {
        let key =
            EncodingKey::from_rsa_pem(include_bytes!("../tests/fixtures/test-signing-key.pem"))
                .unwrap();
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(KID.to_owned());
        encode(&header, claims, &key).unwrap()
    }

    fn without(key: &str) -> Value {
        let mut c = claims();
        c.as_object_mut().unwrap().remove(key);
        c
    }

    fn with(key: &str, value: Value) -> Value {
        let mut c = claims();
        c[key] = value;
        c
    }

    #[tokio::test]
    async fn a_valid_access_token_is_accepted() {
        let user = keycloak().verify(&sign(&claims())).await.unwrap();
        assert_eq!(user.user_id.as_str(), "user-1");
        assert_eq!(user.roles, vec![Role::Admin]);
    }

    #[tokio::test]
    async fn issuer_and_audience_are_required_and_checked() {
        let verifier = keycloak();
        for (name, claims) in [
            ("iss なし", without("iss")),
            ("aud なし", without("aud")),
            ("exp なし", without("exp")),
            ("別の iss", with("iss", json!("http://evil.example.com/realms/platform"))),
            ("別の aud", with("aud", json!("account"))),
            ("期限切れ", with("exp", json!(now() - 300))),
            ("まだ使えない", with("nbf", json!(now() + 300))),
            ("ID トークン", with("typ", json!("ID"))),
        ] {
            assert!(
                matches!(verifier.verify(&sign(&claims)).await, Err(AuthError::InvalidToken(_))),
                "{name} は拒否されるはず"
            );
        }
    }

    #[tokio::test]
    async fn tokens_not_signed_with_the_published_key_are_rejected() {
        let verifier = keycloak();

        // 共通鍵(HS256)で作ったトークン。公開鍵を共通鍵として使わせる攻撃を含め、RS256 以外は受け付けない
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some(KID.to_owned());
        let hs256 = encode(&header, &claims(), &EncodingKey::from_secret(b"secret")).unwrap();
        assert!(matches!(verifier.verify(&hs256).await, Err(AuthError::InvalidToken(_))));

        // 知らない kid(直前に鍵を取り直したばかりなので、取りに行かずに拒否する)
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("unknown".to_owned());
        let key =
            EncodingKey::from_rsa_pem(include_bytes!("../tests/fixtures/test-signing-key.pem"))
                .unwrap();
        let unknown = encode(&header, &claims(), &key).unwrap();
        assert!(matches!(verifier.verify(&unknown).await, Err(AuthError::InvalidToken(_))));
    }
}
