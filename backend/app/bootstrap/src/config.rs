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
    pub url: String,
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
    pub keycloak_admin_client_secret: String,
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
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct SecretsConfig {
    pub database_url_secret_id: String,
    pub payout_api_key_secret_id: String,
}

#[derive(Debug, Deserialize)]
pub struct TelemetryConfig {
    pub otlp_endpoint: String,
    pub json_logs: bool,
    pub log_filter: String,
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
