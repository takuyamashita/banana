//! 設定の読み込みと依存の組み立て。
//!
//! server・Lambda・migrate はここを経由して同じユースケースを組み立てる。
//! domain・usecase・infrastructure は環境変数もファイルも読まず、型付きの値を注入されるだけ。

mod config;

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use payroll_handler::{
    PayrollServiceHandler, ProjectServiceHandler, StaffServiceHandler, UserServiceHandler,
};
use payroll_infrastructure::clock::SystemClock;
use payroll_infrastructure::database::MySqlDatabase;
use payroll_infrastructure::external::{
    BankPayoutGateway, CognitoUserDirectory, KeycloakUserDirectory, LoggingPayoutGateway,
};
use payroll_infrastructure::messaging::outbox::MySqlEventOutbox;
use payroll_infrastructure::query::{MySqlProjectQuery, MySqlStaffQuery};
use payroll_infrastructure::repository::{
    MySqlPayoutRepository, MySqlPayslipRepository, MySqlProjectRepository, MySqlStaffRepository,
};
use payroll_usecase::payslip::{
    CreatePayslipUseCase, FinalizePayslipUseCase, GetPayslipUseCase, ListPayslipsUseCase,
    RequestPayoutUseCase,
};
use payroll_usecase::ports::database::Database;
use payroll_usecase::ports::events::EventOutbox;
use payroll_usecase::ports::payout_gateway::PayoutGateway;
use payroll_usecase::ports::queries::{ProjectQuery, StaffQuery};
use payroll_usecase::ports::repository::{PayslipRepository, ProjectRepository, StaffRepository};
use payroll_usecase::ports::user_directory::UserDirectory;
use payroll_usecase::project::{CreateProjectUseCase, ListProjectsUseCase};
use payroll_usecase::staff::{CreateStaffUseCase, GetMeUseCase, ListStaffUseCase};
use payroll_usecase::user::CreateAdminUserUseCase;
use platform_auth::{ClaimMapper, OidcVerifier};
use sqlx::MySqlPool;

pub use crate::config::{AppConfig, AuthProvider, PayoutProvider, load_config};

pub use platform_telemetry::TelemetryGuard;

pub fn init_telemetry(config: &AppConfig, service_name: &str) -> anyhow::Result<TelemetryGuard> {
    let otlp = Some(config.telemetry.otlp_endpoint.as_str()).filter(|s| !s.is_empty());
    platform_telemetry::init(&platform_telemetry::TelemetryConfig {
        service_name,
        environment: &config.env,
        service_version: env!("CARGO_PKG_VERSION"),
        sample_ratio: config.telemetry.sample_ratio,
        otlp_endpoint: otlp,
        json_logs: config.telemetry.json_logs,
        default_filter: &config.telemetry.log_filter,
    })
    .map_err(|e| anyhow::anyhow!("failed to init telemetry: {e}"))
}

pub async fn aws_config() -> aws_config::SdkConfig {
    aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await
}

/// 秘密の値を読む。シークレット ID の指定があれば Secrets Manager から、なければ設定の値をそのまま使う
async fn secret_or(
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

/// DB の接続文字列。Secrets Manager の指定があればそちらを優先する
pub async fn database_url(
    config: &AppConfig,
    aws: &aws_config::SdkConfig,
) -> anyhow::Result<String> {
    let url = secret_or(
        aws,
        &config.secrets.database_url_secret_id,
        config.database.url.expose(),
        "database url",
    )
    .await?;
    anyhow::ensure!(!url.is_empty(), "database.url is not configured");
    Ok(url)
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

/// DB 接続。接続文字列は Secrets Manager 指定があればそちらを優先する
pub async fn connect_db(
    config: &AppConfig,
    aws: &aws_config::SdkConfig,
) -> anyhow::Result<MySqlPool> {
    let url = database_url(config, aws).await?;
    payroll_infrastructure::connect(&url, config.database.max_connections)
        .await
        .context("failed to connect to database")
}

/// SQS のクライアント。呼び出しにタイムアウトを付ける(SDK の既定は接続のタイムアウトだけで、
/// 応答が返らないと relay がトランザクションと接続を持ったまま待ち続ける)
pub fn sqs_client(config: &AppConfig, aws: &aws_config::SdkConfig) -> aws_sdk_sqs::Client {
    let timeouts = aws_sdk_sqs::config::timeout::TimeoutConfig::builder()
        .connect_timeout(Duration::from_secs(2))
        .operation_attempt_timeout(Duration::from_secs(5))
        .operation_timeout(Duration::from_secs(10))
        .build();
    let mut builder = aws_sdk_sqs::config::Builder::from(aws).timeout_config(timeouts);
    if !config.messaging.sqs_endpoint.is_empty() {
        builder = builder.endpoint_url(&config.messaging.sqs_endpoint);
    }
    aws_sdk_sqs::Client::from_conf(builder.build())
}

// どちらを注入するかはconfigだけで決まる。usecaseはtraitしか見ない
pub fn build_user_directory(
    config: &AppConfig,
    aws: &aws_config::SdkConfig,
) -> Arc<dyn UserDirectory> {
    let auth = &config.auth;
    match auth.provider {
        AuthProvider::Cognito => Arc::new(CognitoUserDirectory::new(
            aws_sdk_cognitoidentityprovider::Client::new(aws),
            auth.cognito_user_pool_id.clone(),
        )),
        AuthProvider::Keycloak => Arc::new(KeycloakUserDirectory::new(
            http_client(),
            auth.keycloak_base_url.clone(),
            auth.keycloak_realm.clone(),
            auth.keycloak_admin_client_id.clone(),
            auth.keycloak_admin_client_secret.expose().to_owned(),
        )),
    }
}

pub fn build_verifier(config: &AppConfig) -> Arc<OidcVerifier> {
    let auth = &config.auth;
    let mapper = match auth.provider {
        AuthProvider::Keycloak => ClaimMapper::Keycloak { audience: auth.audience.clone() },
        AuthProvider::Cognito => ClaimMapper::Cognito { client_id: auth.cognito_client_id.clone() },
    };
    Arc::new(OidcVerifier::new(http_client(), auth.issuer.clone(), mapper))
}

pub struct Handlers {
    pub payroll: PayrollServiceHandler,
    pub staff: StaffServiceHandler,
    pub project: ProjectServiceHandler,
    pub user: UserServiceHandler,
}

pub fn build_handlers(pool: &MySqlPool, user_directory: Arc<dyn UserDirectory>) -> Handlers {
    let payslips: Arc<dyn PayslipRepository> = Arc::new(MySqlPayslipRepository::new(pool.clone()));
    let staff: Arc<dyn StaffRepository> = Arc::new(MySqlStaffRepository::new(pool.clone()));
    let projects: Arc<dyn ProjectRepository> = Arc::new(MySqlProjectRepository::new(pool.clone()));
    let outbox: Arc<dyn EventOutbox> = Arc::new(MySqlEventOutbox);
    let db: Arc<dyn Database> = Arc::new(MySqlDatabase::new(pool.clone()));
    let staff_query: Arc<dyn StaffQuery> = Arc::new(MySqlStaffQuery::new(pool.clone()));
    let project_query: Arc<dyn ProjectQuery> = Arc::new(MySqlProjectQuery::new(pool.clone()));

    Handlers {
        payroll: PayrollServiceHandler::new(
            CreatePayslipUseCase::new(
                payslips.clone(),
                staff.clone(),
                projects.clone(),
                db.clone(),
            ),
            FinalizePayslipUseCase::new(
                payslips.clone(),
                outbox,
                db.clone(),
                Arc::new(SystemClock),
            ),
            GetPayslipUseCase::new(payslips.clone(), staff.clone()),
            ListPayslipsUseCase::new(payslips, staff.clone()),
        ),
        staff: StaffServiceHandler::new(
            CreateStaffUseCase::new(staff.clone(), db.clone(), user_directory.clone()),
            ListStaffUseCase::new(staff_query),
            GetMeUseCase::new(staff),
        ),
        project: ProjectServiceHandler::new(
            CreateProjectUseCase::new(projects, db),
            ListProjectsUseCase::new(project_query),
        ),
        user: UserServiceHandler::new(CreateAdminUserUseCase::new(user_directory)),
    }
}

/// 認証基盤(JWKS・Keycloak の管理 API)への HTTP クライアント。
/// タイムアウトがないと、認証基盤が応答しないときにリクエストが待ち続ける
fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(5))
        .build()
        // 失敗するのは TLS の初期化に失敗したときだけ。そのときは既定のクライアントで続ける
        .unwrap_or_default()
}

/// 振込先の応答を待つ上限。1件あたりの上限 × バッチの件数が Lambda のタイムアウトに収まるようにする
const PAYOUT_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const PAYOUT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

// 振込はSQSのconsumer(Lambda)側で組み立てる。振込 API のキーは Secrets Manager 指定があればそちらを使う
pub async fn build_request_payout(
    config: &AppConfig,
    aws: &aws_config::SdkConfig,
    pool: &MySqlPool,
) -> anyhow::Result<RequestPayoutUseCase> {
    let gateway: Arc<dyn PayoutGateway> = match config.payout.provider {
        PayoutProvider::Logging => Arc::new(LoggingPayoutGateway),
        PayoutProvider::Bank => {
            let api_key = secret_or(
                aws,
                &config.secrets.payout_api_key_secret_id,
                config.payout.api_key.expose(),
                "payout api key",
            )
            .await?;
            anyhow::ensure!(!api_key.is_empty(), "payout api key is not configured");
            Arc::new(BankPayoutGateway::new(
                reqwest::Client::builder()
                    .connect_timeout(PAYOUT_CONNECT_TIMEOUT)
                    .timeout(PAYOUT_REQUEST_TIMEOUT)
                    .build()
                    .context("failed to build the payout HTTP client")?,
                config.payout.base_url.clone(),
                api_key,
            ))
        }
    };
    Ok(RequestPayoutUseCase::new(
        Arc::new(MySqlPayoutRepository::new(pool.clone())),
        gateway,
        Arc::new(MySqlDatabase::new(pool.clone())),
    ))
}
