//! 勤怠サービスの設定の読み込みと依存の組み立て。
//!
//! server・migrate はここを経由して同じユースケースを組み立てる。
//! domain・usecase・infrastructure は環境変数もファイルも読まず、型付きの値を注入されるだけ。

mod config;

use std::sync::Arc;
use std::time::Duration;

use platform_gen::acme::timesheet::v1::timesheet_service_server::TimesheetServiceServer;
use platform_messaging::relay::RelayLock;
use platform_service::server::{MAX_MESSAGE_BYTES, Started};
use sqlx::MySqlPool;
use timesheet_handler::{PayrollEventHandler, TimesheetServiceHandler};
use timesheet_infrastructure::clock::SystemClock;
use timesheet_infrastructure::database::MySqlDatabase;
use timesheet_infrastructure::messaging::outbox::MySqlEventOutbox;
use timesheet_infrastructure::repository::{
    MySqlProjectRepository, MySqlStaffRepository, MySqlTimesheetRepository,
};
use timesheet_usecase::approval::{
    ApproveTimesheetUseCase, ListSubmittedTimesheetsUseCase, ReturnTimesheetUseCase,
};
use timesheet_usecase::copies::{ListProjectsUseCase, RecordProjectUseCase, RecordStaffUseCase};
use timesheet_usecase::my_timesheet::{
    GetMyTimesheetUseCase, SaveMyTimesheetUseCase, SubmitMyTimesheetUseCase,
};
use timesheet_usecase::ports::clock::Clock;
use timesheet_usecase::ports::database::Database;
use timesheet_usecase::ports::events::EventOutbox;
use timesheet_usecase::ports::repository::{
    ProjectRepository, StaffRepository, TimesheetRepository,
};
use tokio_util::sync::CancellationToken;
use tonic::service::Routes;

pub use crate::config::{AppConfig, load_config};
pub use platform_service::{TelemetryGuard, aws_config};

/// サービスの名前。トレース・relay のロックの名前に使う
pub const SERVICE: &str = "timesheet";

pub fn init_telemetry(config: &AppConfig, name: &str) -> anyhow::Result<TelemetryGuard> {
    platform_service::init_telemetry(&config.env, &config.telemetry, name)
}

/// マイグレーションを流す(timesheet-migrate)
pub async fn migrate(config: &AppConfig) -> anyhow::Result<()> {
    let aws = aws_config().await;
    platform_service::migrate::run(
        &config.database,
        &config.secrets,
        &aws,
        &timesheet_infrastructure::MIGRATOR,
    )
    .await
}

/// 記録の読み書きに使うもの
struct Deps {
    timesheets: Arc<dyn TimesheetRepository>,
    staff: Arc<dyn StaffRepository>,
    projects: Arc<dyn ProjectRepository>,
    db: Arc<dyn Database>,
}

impl Deps {
    fn new(pool: &MySqlPool) -> Self {
        Self {
            timesheets: Arc::new(MySqlTimesheetRepository::new(pool.clone())),
            staff: Arc::new(MySqlStaffRepository::new(pool.clone())),
            projects: Arc::new(MySqlProjectRepository::new(pool.clone())),
            db: Arc::new(MySqlDatabase::new(pool.clone())),
        }
    }
}

/// gRPC のハンドラを組み立てる
pub fn build_handler(pool: &MySqlPool) -> TimesheetServiceHandler {
    let Deps { timesheets, staff, projects, db } = Deps::new(pool);
    let outbox: Arc<dyn EventOutbox> = Arc::new(MySqlEventOutbox);
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    TimesheetServiceHandler::new(
        GetMyTimesheetUseCase::new(timesheets.clone(), staff.clone()),
        SaveMyTimesheetUseCase::new(
            timesheets.clone(),
            staff.clone(),
            projects.clone(),
            db.clone(),
        ),
        SubmitMyTimesheetUseCase::new(timesheets.clone(), staff.clone(), db.clone(), clock.clone()),
        ListProjectsUseCase::new(projects),
        ListSubmittedTimesheetsUseCase::new(timesheets.clone(), staff.clone()),
        ApproveTimesheetUseCase::new(timesheets.clone(), staff.clone(), outbox, db.clone(), clock),
        ReturnTimesheetUseCase::new(timesheets, staff, db),
    )
}

/// 給与から届く出来事の受け手を組み立てる
pub fn build_event_handler(pool: &MySqlPool) -> PayrollEventHandler {
    let Deps { staff, projects, db, .. } = Deps::new(pool);
    PayrollEventHandler::new(
        RecordStaffUseCase::new(staff, db.clone()),
        RecordProjectUseCase::new(projects, db),
    )
}

/// server を組み立てる。設定を確かめ、DB に接続し、gRPC のサービス・relay・受け手を用意する
pub async fn start_server(
    config: &AppConfig,
    cancel: CancellationToken,
) -> anyhow::Result<Started> {
    config.validate_for_server()?;
    let aws = aws_config().await;
    let pool = platform_service::connect_db(&config.database, &config.secrets, &aws).await?;

    let grpc = Routes::new(
        TimesheetServiceServer::new(build_handler(&pool))
            .max_decoding_message_size(MAX_MESSAGE_BYTES),
    );

    // outbox relay(承認した勤務表を給与に知らせる)と、給与から届く出来事の受け手を常駐タスクとして動かす
    let relay = tokio::spawn(platform_messaging::relay::run(
        pool.clone(),
        platform_service::build_publisher(&config.messaging, &aws)?,
        RelayLock::for_service(SERVICE),
        Duration::from_millis(config.messaging.relay_interval_ms),
        cancel.clone(),
    ));
    let consumer = tokio::spawn(platform_messaging::consumer::run(
        platform_messaging::sqs_client(&aws, &config.messaging.sqs_endpoint),
        config.messaging.inbox_queue_url.clone(),
        build_event_handler(&pool),
        cancel,
    ));

    Ok(Started {
        grpc,
        verifier: platform_service::build_verifier(&config.auth),
        pool,
        tasks: vec![relay, consumer],
    })
}
