//! サービスの共通の組み立て。
//!
//! どのサービスの server・migrate・Lambda も、ここを経由して設定を読み、シークレットを取り、DB に接続し、
//! 同じ決まりで起動して止まる。サービスごとの違い(ユースケースの組み立て・業務の設定)は各サービスの
//! bootstrap に置く

pub mod config;
pub mod migrate;
pub mod server;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use platform_auth::{ClaimMapper, OidcVerifier};
use platform_messaging::publisher::{Publisher, SnsPublisher, SqsFanoutPublisher};
use sqlx::MySqlPool;

use crate::config::{
    AuthConfig, AuthProvider, DatabaseConfig, MessagingConfig, SecretsConfig, TelemetryConfig,
};

pub use platform_telemetry::TelemetryGuard;

/// ログとトレースを初期化する。`service_name` はトレースに出るサービスの名前(payroll-server など)
pub fn init_telemetry(
    env: &str,
    telemetry: &TelemetryConfig,
    service_name: &str,
) -> anyhow::Result<TelemetryGuard> {
    let otlp = Some(telemetry.otlp_endpoint.as_str()).filter(|s| !s.is_empty());
    platform_telemetry::init(&platform_telemetry::TelemetryConfig {
        service_name,
        environment: env,
        service_version: env!("CARGO_PKG_VERSION"),
        sample_ratio: telemetry.sample_ratio,
        otlp_endpoint: otlp,
        json_logs: telemetry.json_logs,
        default_filter: &telemetry.log_filter,
    })
    .map_err(|e| anyhow::anyhow!("failed to init telemetry: {e}"))
}

pub async fn aws_config() -> aws_config::SdkConfig {
    aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await
}

/// 秘密の値を読む。シークレット ID の指定があれば Secrets Manager から、なければ設定の値をそのまま使う
pub async fn secret_or(
    aws: &aws_config::SdkConfig,
    secret_id: &str,
    fallback: &str,
    what: &str,
) -> anyhow::Result<String> {
    if secret_id.is_empty() {
        return Ok(fallback.to_owned());
    }
    Ok(aws_sdk_secretsmanager::Client::new(aws)
        .get_secret_value()
        .secret_id(secret_id)
        .send()
        .await
        .with_context(|| format!("failed to read {what} secret"))?
        .secret_string()
        .with_context(|| format!("{what} secret is not a string"))?
        .to_owned())
}

/// Secrets Manager のシークレットを読む
pub async fn read_secret(
    aws: &aws_config::SdkConfig,
    secret_id: &str,
    what: &str,
) -> anyhow::Result<String> {
    anyhow::ensure!(!secret_id.is_empty(), "{what} secret id is not configured");
    secret_or(aws, secret_id, "", what).await
}

/// DB の接続文字列。Secrets Manager の指定があればそちらを優先する
pub async fn database_url(
    database: &DatabaseConfig,
    secrets: &SecretsConfig,
    aws: &aws_config::SdkConfig,
) -> anyhow::Result<String> {
    let url =
        secret_or(aws, &secrets.database_url_secret_id, database.url.expose(), "database url")
            .await?;
    anyhow::ensure!(!url.is_empty(), "database.url is not configured");
    Ok(url)
}

/// DB に接続する
pub async fn connect_db(
    database: &DatabaseConfig,
    secrets: &SecretsConfig,
    aws: &aws_config::SdkConfig,
) -> anyhow::Result<MySqlPool> {
    let url = database_url(database, secrets, aws).await?;
    platform_db::connect(&url, database.max_connections)
        .await
        .context("failed to connect to database")
}

/// アクセストークンの検証。認証基盤ごとのクレームの差は設定だけで決まる
pub fn build_verifier(auth: &AuthConfig) -> Arc<OidcVerifier> {
    let mapper = match auth.provider {
        AuthProvider::Keycloak => ClaimMapper::Keycloak { audience: auth.audience.clone() },
        AuthProvider::Cognito => ClaimMapper::Cognito { client_id: auth.cognito_client_id.clone() },
    };
    Arc::new(OidcVerifier::new(http_client(), auth.issuer.clone(), mapper))
}

/// 出来事の送り先。トピックの指定があれば SNS(AWS)、なければ受け手のキューへ直接送る(ローカル)
pub fn build_publisher(
    messaging: &MessagingConfig,
    aws: &aws_config::SdkConfig,
) -> anyhow::Result<Arc<dyn Publisher>> {
    if !messaging.topic_arn.is_empty() {
        return Ok(Arc::new(SnsPublisher::new(
            platform_messaging::sns_client(aws),
            messaging.topic_arn.clone(),
        )));
    }
    anyhow::ensure!(
        !messaging.publish_queue_urls.is_empty(),
        "messaging.topic_arn か messaging.publish_queue_urls を指定してください"
    );
    Ok(Arc::new(SqsFanoutPublisher::new(
        platform_messaging::sqs_client(aws, &messaging.sqs_endpoint),
        messaging.publish_queue_urls.clone(),
    )))
}

/// 外(認証基盤の JWKS・管理 API など)への HTTP クライアント。
/// タイムアウトがないと、相手が応答しないときにリクエストが待ち続ける
pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(5))
        .build()
        // 失敗するのは TLS の初期化に失敗したときだけ。そのときは既定のクライアントで続ける
        .unwrap_or_default()
}
