//! OIDC のアクセストークン検証。
//!
//! 署名鍵(JWKS)は kid ごとにキャッシュし、知らない kid が来たら取り直す(鍵ローテーションへの追従)。
//! プロバイダごとのクレームの差は [`ClaimMapper`] に閉じ込め、外には `AuthenticatedUser` だけを出す。

mod jwks;
mod mapper;

use std::time::Duration;

use jsonwebtoken::{Algorithm, Validation, decode, decode_header};
use platform_kernel::AuthenticatedUser;
use thiserror::Error;

pub use mapper::ClaimMapper;

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

    pub async fn verify(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        let header = decode_header(token).map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        let kid = header.kid.ok_or_else(|| AuthError::InvalidToken("kid missing".into()))?;
        let key = self.jwks.key(&kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        self.mapper.configure(&mut validation);

        let data = decode::<serde_json::Value>(token, &key, &validation)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        self.mapper.map(&data.claims)
    }
}
