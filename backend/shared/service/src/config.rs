//! 設定の読み込みと、どのサービスにもある設定の節。
//!
//! 設定はサービスごとの `config/<サービス>/default.toml` に `{APP_ENV}.toml` と環境変数
//! `<サービス>__SECTION__KEY`(例: `PAYROLL__DATABASE__URL`)を重ねて作る。環境変数の接頭辞を
//! サービスごとに分けるのは、同じシェル(ローカルの .env)から起動する別のサービスに値が混ざらないようにするため。
//! toml はバイナリに埋め込む(Lambda の zip のように、設定ファイルを置けない配り方でも動くようにする)。
//! `APP_CONFIG_DIR` を指定したときだけファイルを読む

use anyhow::Context as _;
use serde::Deserialize;
use serde::de::DeserializeOwned;

/// バイナリに埋め込んだ設定。`default` に、環境(local/dev/stg/prd)ごとの toml を重ねる
pub struct EmbeddedConfig {
    /// 上書きする環境変数の接頭辞(PAYROLL なら PAYROLL__SECTION__KEY)
    pub env_prefix: &'static str,
    pub default: &'static str,
    pub envs: &'static [(&'static str, &'static str)],
}

/// 環境変数で与えるリスト(カンマ区切り)の設定
const LIST_KEYS: [&str; 2] = ["server.cors_allowed_origins", "messaging.publish_queue_urls"];

/// `APP_ENV` を見て、埋め込みの `default` に `{env}.toml` と環境変数を重ねて読む
pub fn load<T: DeserializeOwned>(embedded: &EmbeddedConfig) -> anyhow::Result<T> {
    let env = std::env::var("APP_ENV").context("APP_ENV is not set (local/dev/stg/prd)")?;
    let env_toml = embedded
        .envs
        .iter()
        .find(|(name, _)| *name == env)
        .map(|(_, toml)| *toml)
        .with_context(|| format!("unknown APP_ENV: {env}"))?;
    let builder = config::Config::builder()
        .add_source(config::File::from_str(embedded.default, config::FileFormat::Toml));
    let builder = match std::env::var("APP_CONFIG_DIR") {
        Ok(dir) => {
            builder.add_source(config::File::with_name(&format!("{dir}/{env}")).required(true))
        }
        Err(_) => builder.add_source(config::File::from_str(env_toml, config::FileFormat::Toml)),
    };
    let mut environment = config::Environment::with_prefix(embedded.env_prefix)
        .prefix_separator("__")
        .separator("__")
        .list_separator(",")
        .try_parsing(true);
    for key in LIST_KEYS {
        environment = environment.with_list_parse_key(key);
    }
    builder
        .add_source(environment)
        .set_override("env", env)?
        .build()?
        .try_deserialize()
        .context("invalid configuration")
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
pub struct ServerConfig {
    pub addr: String,
    pub cors_allowed_origins: Vec<String>,
    pub shutdown_grace_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    /// 接続文字列(シークレット)。AWS では secrets.database_url_secret_id で与える
    pub url: Secret,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthProvider {
    Keycloak,
    Cognito,
}

/// アクセストークンの検証
#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub provider: AuthProvider,
    pub issuer: String,
    /// keycloak: aud に入る API 識別子
    pub audience: String,
    /// cognito: アプリクライアント ID
    pub cognito_client_id: String,
}

/// サービスの間の出来事の送受信
#[derive(Debug, Deserialize)]
pub struct MessagingConfig {
    /// 出来事を出す SNS の FIFO トピック(AWS)
    pub topic_arn: String,
    /// トピックがないとき(ローカル)に、出来事を直接送る受け手のキュー
    pub publish_queue_urls: Vec<String>,
    /// このサービスが受け取る出来事のキュー。空なら受け取らない
    pub inbox_queue_url: String,
    /// ローカル(ElasticMQ)だけ指定する
    pub sqs_endpoint: String,
    pub relay_interval_ms: u64,
}

#[derive(Debug, Deserialize)]
#[allow(
    clippy::struct_field_names,
    reason = "設定のキー(<サービス>__SECRETS__*_SECRET_ID)と同じ名前にする"
)]
pub struct SecretsConfig {
    /// 指定すると起動時に Secrets Manager から接続文字列を取る(AWS では Terraform が環境変数で渡す)
    pub database_url_secret_id: String,
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
    /// 新しく始まるトレースのうち送る割合(0.0〜1.0)
    pub sample_ratio: f64,
}

/// 起動に要る値がそろっているかを確かめる。空や REPLACE_ME のまま起動すると、/health は通るのに
/// 全リクエストが失敗するので、足りなければ起動時に止める
#[derive(Default)]
pub struct Required(Vec<&'static str>);

impl Required {
    pub fn value(&mut self, name: &'static str, value: &str) -> &mut Self {
        if value.trim().is_empty() || value.contains("REPLACE_ME") {
            self.0.push(name);
        }
        self
    }

    /// server が要る共通の値(認証・CORS・DB の接続先・出来事の送り先)
    pub fn server(
        &mut self,
        server: &ServerConfig,
        database: &DatabaseConfig,
        secrets: &SecretsConfig,
        auth: &AuthConfig,
        messaging: &MessagingConfig,
    ) -> &mut Self {
        self.value("auth.issuer", &auth.issuer);
        if auth.provider == AuthProvider::Cognito {
            self.value("auth.cognito_client_id", &auth.cognito_client_id);
        }
        if server.cors_allowed_origins.is_empty() {
            self.0.push("server.cors_allowed_origins");
        }
        if database.url.expose().is_empty() && secrets.database_url_secret_id.is_empty() {
            self.0.push("database.url か secrets.database_url_secret_id");
        }
        if messaging.topic_arn.is_empty() && messaging.publish_queue_urls.is_empty() {
            self.0.push("messaging.topic_arn か messaging.publish_queue_urls");
        }
        self
    }

    pub fn check(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.0.is_empty(), "設定が足りません: {}", self.0.join(", "));
        Ok(())
    }
}
