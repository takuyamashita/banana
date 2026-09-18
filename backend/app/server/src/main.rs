mod app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = bootstrap::load_config()?;
    let _telemetry = bootstrap::init_telemetry(&config, "server")?;
    app::run(config).await
}
