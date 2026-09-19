//! 給与(payroll)サービスの gRPC(+ gRPC-Web)サーバー。outbox relay も常駐タスクとして動かす

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = payroll_bootstrap::load_config()?;
    let _telemetry = payroll_bootstrap::init_telemetry(&config, "payroll-server")?;
    platform_service::server::run(&config.server, |cancel| {
        payroll_bootstrap::start_server(&config, cancel)
    })
    .await
}
