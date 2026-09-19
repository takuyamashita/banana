//! 設定の読み込みと依存の組み立て。
//!
//! server・Lambda・migrate はここを経由して同じユースケースを組み立てる。
//! domain・usecase・infrastructure は環境変数もファイルも読まず、型付きの値を注入されるだけ。

mod config;

use std::sync::Arc;
use std::time::Duration;

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
use platform_gen::acme::payroll::v1::payroll_service_server::PayrollServiceServer;
use platform_gen::acme::payroll::v1::project_service_server::ProjectServiceServer;
use platform_gen::acme::payroll::v1::staff_service_server::StaffServiceServer;
use platform_messaging::relay::RelayLock;
use platform_service::config::AuthProvider;
use platform_service::server::{MAX_MESSAGE_BYTES, Started};
use sqlx::MySqlPool;
use tokio_util::sync::CancellationToken;
use tonic::service::Routes;

pub use crate::config::{AppConfig, PayoutProvider, load_config};
pub use platform_service::{TelemetryGuard, aws_config};

/// サービスの名前。トレース・relay のロックの名前に使う
pub const SERVICE: &str = "payroll";

pub fn init_telemetry(config: &AppConfig, name: &str) -> anyhow::Result<TelemetryGuard> {
    platform_service::init_telemetry(&config.env, &config.telemetry, name)
}

/// DB に接続する。接続文字列は Secrets Manager 指定があればそちらを優先する
pub async fn connect_db(
    config: &AppConfig,
    aws: &aws_config::SdkConfig,
) -> anyhow::Result<MySqlPool> {
    platform_service::connect_db(&config.database, &config.secrets, aws).await
}

/// マイグレーションを流す(payroll-migrate)
pub async fn migrate(config: &AppConfig) -> anyhow::Result<()> {
    let aws = aws_config().await;
    platform_service::migrate::run(
        &config.database,
        &config.secrets,
        &aws,
        &payroll_infrastructure::MIGRATOR,
    )
    .await
}

/// server を組み立てる。設定を確かめ、DB に接続し、gRPC のサービスと relay を用意する
pub async fn start_server(
    config: &AppConfig,
    cancel: CancellationToken,
) -> anyhow::Result<Started> {
    config.validate_for_server()?;
    let aws = aws_config().await;
    let pool = connect_db(config, &aws).await?;
    let handlers = build_handlers(&pool, build_user_directory(config, &aws));

    let grpc = Routes::new(
        PayrollServiceServer::new(handlers.payroll).max_decoding_message_size(MAX_MESSAGE_BYTES),
    )
    .add_service(
        StaffServiceServer::new(handlers.staff).max_decoding_message_size(MAX_MESSAGE_BYTES),
    )
    .add_service(
        ProjectServiceServer::new(handlers.project).max_decoding_message_size(MAX_MESSAGE_BYTES),
    );

    // outbox relay を常駐タスクとして動かす。複数インスタンスでも、送るのはロックを取れた1つだけ
    let relay = tokio::spawn(platform_messaging::relay::run(
        pool.clone(),
        platform_service::build_publisher(&config.messaging, &aws)?,
        RelayLock::for_service(SERVICE),
        Duration::from_millis(config.messaging.relay_interval_ms),
        cancel,
    ));

    Ok(Started {
        grpc,
        verifier: platform_service::build_verifier(&config.auth),
        pool,
        tasks: vec![relay],
    })
}

// どちらを注入するかはconfigだけで決まる。usecaseはtraitしか見ない
pub fn build_user_directory(
    config: &AppConfig,
    aws: &aws_config::SdkConfig,
) -> Arc<dyn UserDirectory> {
    let directory = &config.user_directory;
    match config.auth.provider {
        AuthProvider::Cognito => Arc::new(CognitoUserDirectory::new(
            aws_sdk_cognitoidentityprovider::Client::new(aws),
            directory.cognito_user_pool_id.clone(),
        )),
        AuthProvider::Keycloak => Arc::new(KeycloakUserDirectory::new(
            platform_service::http_client(),
            directory.keycloak_base_url.clone(),
            directory.keycloak_realm.clone(),
            directory.keycloak_admin_client_id.clone(),
            directory.keycloak_admin_client_secret.expose().to_owned(),
        )),
    }
}

pub struct Handlers {
    pub payroll: PayrollServiceHandler,
    pub staff: StaffServiceHandler,
    pub project: ProjectServiceHandler,
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
                outbox.clone(),
                db.clone(),
                Arc::new(SystemClock),
            ),
            GetPayslipUseCase::new(payslips.clone(), staff.clone()),
            ListPayslipsUseCase::new(payslips, staff.clone()),
        ),
        staff: StaffServiceHandler::new(
            CreateStaffUseCase::new(staff.clone(), outbox.clone(), db.clone(), user_directory),
            ListStaffUseCase::new(staff_query),
            GetMeUseCase::new(staff),
        ),
        project: ProjectServiceHandler::new(
            CreateProjectUseCase::new(projects, outbox, db),
            ListProjectsUseCase::new(project_query),
        ),
    }
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
            let api_key = platform_service::secret_or(
                aws,
                &config.payout.api_key_secret_id,
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
