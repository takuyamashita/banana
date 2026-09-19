use anyhow::Context as _;
use serde::Deserialize;

const DEFAULT_TOML: &str = include_str!("../../../../config/default.toml");

/// 環境ごとの設定もバイナリに埋め込む。Lambda の zip のように、設定ファイルを置けない配り方でも動くようにする
const ENV_TOMLS: [(&str, &str); 4] = [
    ("local", include_str!("../../../../config/local.toml")),
    ("dev", include_str!("../../../../config/dev.toml")),
    ("stg", include_str!("../../../../config/stg.toml")),
    ("prd", include_str!("../../../../config/prd.toml")),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthProvider {
    Keycloak,
    Cognito,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PayoutProvider {
    Logging,
    Bank,
}

/// 秘密の値(接続文字列・クライアントシークレット・API キー)。
/// `{:?}` でログに出しても中身が見えないようにする。中身は `expose` で明示して取り出す
#[derive(Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0.is_empty() { "\"\"" } else { "***" })
    }
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub env: String,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub messaging: MessagingConfig,
    pub payout: PayoutConfig,
    pub secrets: SecretsConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub addr: String,
    pub cors_allowed_origins: Vec<String>,
    pub shutdown_grace_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: Secret,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub provider: AuthProvider,
    pub issuer: String,
    pub audience: String,
    pub cognito_client_id: String,
    pub cognito_user_pool_id: String,
    pub keycloak_base_url: String,
    pub keycloak_realm: String,
    pub keycloak_admin_client_id: String,
    pub keycloak_admin_client_secret: Secret,
}

#[derive(Debug, Deserialize)]
pub struct MessagingConfig {
    pub queue_url: String,
    pub sqs_endpoint: String,
    pub relay_interval_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct PayoutConfig {
    pub provider: PayoutProvider,
    pub base_url: String,
    pub api_key: Secret,
}

#[derive(Debug, Deserialize)]
#[allow(
    clippy::struct_field_names,
    reason = "設定のキー(APP__SECRETS__*_SECRET_ID)と同じ名前にする"
)]
pub struct SecretsConfig {
    pub database_url_secret_id: String,
    pub payout_api_key_secret_id: String,
    /// migrate が使う DB の管理者(RDS が管理するシークレット。JSON: username, password)
    pub database_admin_secret_id: String,
    /// migrate が作るアプリ用の DB ユーザー(JSON: username, password)
    pub database_app_user_secret_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TelemetryConfig {
    pub otlp_endpoint: String,
    pub json_logs: bool,
    pub log_filter: String,
    pub sample_ratio: f64,
}

/// `APP_ENV`(local/dev/stg/prd)を見て、埋め込みの default.toml に `{env}.toml` と
/// 環境変数 `APP__SECTION__KEY` を重ねる。
///
/// `{env}.toml` も埋め込みを使う。`APP_CONFIG_DIR` を指定したときだけ、そのディレクトリのファイルを読む
pub fn load_config() -> anyhow::Result<AppConfig> {
    let env = std::env::var("APP_ENV").context("APP_ENV is not set (local/dev/stg/prd)")?;
    let embedded = ENV_TOMLS
        .iter()
        .find(|(name, _)| *name == env)
        .map(|(_, toml)| *toml)
        .with_context(|| format!("unknown APP_ENV: {env}"))?;
    let builder = config::Config::builder()
        .add_source(config::File::from_str(DEFAULT_TOML, config::FileFormat::Toml));
    let builder = match std::env::var("APP_CONFIG_DIR") {
        Ok(dir) => {
            builder.add_source(config::File::with_name(&format!("{dir}/{env}")).required(true))
        }
        Err(_) => builder.add_source(config::File::from_str(embedded, config::FileFormat::Toml)),
    };

    builder
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("__")
                .separator("__")
                .list_separator(",")
                .with_list_parse_key("server.cors_allowed_origins")
                .try_parsing(true),
        )
        .set_override("env", env)?
        .build()?
        .try_deserialize()
        .context("invalid configuration")
}

impl AppConfig {
    /// server を動かすのに要る値がそろっているかを確かめる。足りなければ起動時に止める
    /// (空や REPLACE_ME のまま起動すると、/health は通るのに全リクエストが失敗する)
    pub fn validate_for_server(&self) -> anyhow::Result<()> {
        let mut missing = Vec::new();
        let mut require = |name: &'static str, value: &str| {
            if value.trim().is_empty() || value.contains("REPLACE_ME") {
                missing.push(name);
            }
        };
        require("auth.issuer", &self.auth.issuer);
        match self.auth.provider {
            AuthProvider::Cognito => {
                require("auth.cognito_client_id", &self.auth.cognito_client_id);
                require("auth.cognito_user_pool_id", &self.auth.cognito_user_pool_id);
            }
            AuthProvider::Keycloak => {
                require("auth.keycloak_base_url", &self.auth.keycloak_base_url);
                require("auth.keycloak_realm", &self.auth.keycloak_realm);
                require("auth.keycloak_admin_client_id", &self.auth.keycloak_admin_client_id);
                require(
                    "auth.keycloak_admin_client_secret",
                    self.auth.keycloak_admin_client_secret.expose(),
                );
            }
        }
        require("messaging.queue_url", &self.messaging.queue_url);
        if self.server.cors_allowed_origins.is_empty() {
            missing.push("server.cors_allowed_origins");
        }
        if self.database.url.expose().is_empty() && self.secrets.database_url_secret_id.is_empty() {
            missing.push("database.url か secrets.database_url_secret_id");
        }
        anyhow::ensure!(missing.is_empty(), "設定が足りません: {}", missing.join(", "));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(env_toml: &str) -> AppConfig {
        config::Config::builder()
            .add_source(config::File::from_str(DEFAULT_TOML, config::FileFormat::Toml))
            .add_source(config::File::from_str(env_toml, config::FileFormat::Toml))
            .set_override("env", "test")
            .unwrap()
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }

    #[test]
    fn the_local_config_is_complete() {
        let local = ENV_TOMLS.iter().find(|(name, _)| *name == "local").unwrap().1;
        config(local).validate_for_server().unwrap();
    }

    #[test]
    fn missing_or_placeholder_values_stop_the_server() {
        // AWS の環境の toml だけでは、Terraform が渡す値(issuer・キュー URL など)が足りない
        let dev = ENV_TOMLS.iter().find(|(name, _)| *name == "dev").unwrap().1;
        let err = config(dev).validate_for_server().unwrap_err().to_string();
        assert!(err.contains("auth.issuer") && err.contains("messaging.queue_url"), "{err}");

        let placeholder = format!("{dev}\n[messaging]\nqueue_url = \"https://sqs/REPLACE_ME\"\n");
        let err = config(&placeholder).validate_for_server().unwrap_err().to_string();
        assert!(err.contains("messaging.queue_url"), "{err}");
    }

    #[test]
    fn secrets_are_not_shown_in_debug_output() {
        let local = ENV_TOMLS.iter().find(|(name, _)| *name == "local").unwrap().1;
        let shown = format!("{:?}", config(local));
        assert!(!shown.contains("platform-backend-secret"), "{shown}");
        assert!(!shown.contains("mysql://platform:platform"), "{shown}");
    }
}
