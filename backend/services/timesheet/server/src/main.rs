//! 勤怠(timesheet)サービスの gRPC(+ gRPC-Web)サーバー。
//! outbox relay と、給与(payroll)の出来事の受け手も常駐タスクとして動かす

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = timesheet_bootstrap::load_config()?;
    let _telemetry = timesheet_bootstrap::init_telemetry(&config, "timesheet-server")?;
    platform_service::server::run(&config.server, |cancel| {
        timesheet_bootstrap::start_server(&config, cancel)
    })
    .await
}
