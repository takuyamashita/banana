//! 設定の読み込みと依存の組み立て。
//!
//! server・Lambda・migrate はここを経由して同じユースケースを組み立てる。
//! domain・usecase・infrastructure は環境変数もファイルも読まず、型付きの値を注入されるだけ。

mod config;

use std::sync::Arc;

use anyhow::Context as _;
use payroll_handler::{PayrollServiceHandler, ProjectServiceHandler, StaffServiceHandler};
use payroll_infrastructure::clock::SystemClock;
use payroll_infrastructure::database::MySqlDatabase;
use payroll_infrastructure::external::{
    BankPayoutGateway, CognitoUserDirectory, KeycloakUserDirectory, LoggingPayoutGateway,
};
use payroll_infrastructure::messaging::outbox::MySqlEventOutbox;
use payroll_infrastructure::query::{MySqlProjectQuery, MySqlStaffQuery};
use payroll_infrastructure::repository::{
    MySqlPayslipRepository, MySqlProjectRepository, MySqlStaffRepository,
};
use payroll_usecase::payslip::{
    FinalizePayslipUseCase, GetPayslipUseCase, ListPayslipsUseCase, RequestPayoutUseCase,
};
use payroll_usecase::ports::database::Database;
use payroll_usecase::ports::events::EventOutbox;
use payroll_usecase::ports::payout_gateway::PayoutGateway;
use payroll_usecase::ports::queries::{ProjectQuery, StaffQuery};
use payroll_usecase::ports::repository::{PayslipRepository, ProjectRepository, StaffRepository};
use payroll_usecase::ports::user_directory::UserDirectory;
use payroll_usecase::project::{CreateProjectUseCase, ListProjectsUseCase};
use payroll_usecase::staff::{CreateStaffUseCase, GetMeUseCase, ListStaffUseCase};
use platform_auth::{ClaimMapper, OidcVerifier};
use sqlx::MySqlPool;

pub use crate::config::{AppConfig, AuthProvider, PayoutProvider, load_config};

pub use platform_telemetry::TelemetryGuard;

pub fn init_telemetry(config: &AppConfig, service_name: &str) -> anyhow::Result<TelemetryGuard> {
    let otlp = Some(config.telemetry.otlp_endpoint.as_str()).filter(|s| !s.is_empty());
    platform_telemetry::init(&platform_telemetry::TelemetryConfig {
        service_name,
        otlp_endpoint: otlp,
        json_logs: config.telemetry.json_logs,
        default_filter: &config.telemetry.log_filter,
    })
    .map_err(|e| anyhow::anyhow!("failed to init telemetry: {e}"))
}

pub async fn aws_config() -> aws_config::SdkConfig {
    aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await
}

/// DB 接続。接続文字列は Secrets Manager 指定があればそちらを優先する
pub async fn connect_db(
    config: &AppConfig,
    aws: &aws_config::SdkConfig,
) -> anyhow::Result<MySqlPool> {
    let url = if config.secrets.database_url_secret_id.is_empty() {
        config.database.url.clone()
    } else {
        aws_sdk_secretsmanager::Client::new(aws)
            .get_secret_value()
            .secret_id(&config.secrets.database_url_secret_id)
            .send()
            .await
            .context("failed to read database url secret")?
            .secret_string()
            .context("database url secret is not a string")?
            .to_owned()
    };
    anyhow::ensure!(!url.is_empty(), "database.url is not configured");
    payroll_infrastructure::connect(&url, config.database.max_connections)
        .await
        .context("failed to connect to database")
}

pub fn sqs_client(config: &AppConfig, aws: &aws_config::SdkConfig) -> aws_sdk_sqs::Client {
    let mut builder = aws_sdk_sqs::config::Builder::from(aws);
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
            reqwest::Client::new(),
            auth.keycloak_base_url.clone(),
            auth.keycloak_realm.clone(),
            auth.keycloak_admin_client_id.clone(),
            auth.keycloak_admin_client_secret.clone(),
        )),
    }
}

pub fn build_verifier(config: &AppConfig) -> Arc<OidcVerifier> {
    let auth = &config.auth;
    let mapper = match auth.provider {
        AuthProvider::Keycloak => ClaimMapper::Keycloak { audience: auth.audience.clone() },
        AuthProvider::Cognito => ClaimMapper::Cognito { client_id: auth.cognito_client_id.clone() },
    };
    Arc::new(OidcVerifier::new(reqwest::Client::new(), auth.issuer.clone(), mapper))
}

pub struct Handlers {
    pub payroll: PayrollServiceHandler,
    pub staff: StaffServiceHandler,
    pub project: ProjectServiceHandler,
}

pub fn build_handlers(pool: &MySqlPool, user_directory: Arc<dyn UserDirectory>) -> Handlers {
    let payslips: Arc<dyn PayslipRepository> = Arc::new(MySqlPayslipRepository::new(pool.clone()));
    let staff: Arc<dyn StaffRepository> = Arc::new(MySqlStaffRepository::new(pool.clone()));
    let projects: Arc<dyn ProjectRepository> = Arc::new(MySqlProjectRepository);
    let outbox: Arc<dyn EventOutbox> = Arc::new(MySqlEventOutbox);
    let db: Arc<dyn Database> = Arc::new(MySqlDatabase::new(pool.clone()));
    let staff_query: Arc<dyn StaffQuery> = Arc::new(MySqlStaffQuery::new(pool.clone()));
    let project_query: Arc<dyn ProjectQuery> = Arc::new(MySqlProjectQuery::new(pool.clone()));

    Handlers {
        payroll: PayrollServiceHandler::new(
            FinalizePayslipUseCase::new(
                payslips.clone(),
                staff.clone(),
                outbox,
                db.clone(),
                Arc::new(SystemClock),
            ),
            GetPayslipUseCase::new(payslips.clone(), staff.clone()),
            ListPayslipsUseCase::new(payslips, staff.clone()),
        ),
        staff: StaffServiceHandler::new(
            CreateStaffUseCase::new(staff.clone(), db.clone(), user_directory),
            ListStaffUseCase::new(staff_query),
            GetMeUseCase::new(staff),
        ),
        project: ProjectServiceHandler::new(
            CreateProjectUseCase::new(projects, db),
            ListProjectsUseCase::new(project_query),
        ),
    }
}

// 振込はSQSのconsumer(Lambda)側で組み立てる
pub fn build_request_payout(config: &AppConfig) -> RequestPayoutUseCase {
    let gateway: Arc<dyn PayoutGateway> = match config.payout.provider {
        PayoutProvider::Logging => Arc::new(LoggingPayoutGateway),
        PayoutProvider::Bank => Arc::new(BankPayoutGateway::new(
            reqwest::Client::new(),
            config.payout.base_url.clone(),
            config.payout.api_key.clone(),
        )),
    };
    RequestPayoutUseCase::new(gateway)
}
