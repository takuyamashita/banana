//! 適用済みマイグレーションは _sqlx_migrations に記録されるので、再実行しても安全。
//! 失敗したら非0で終了し、デプロイをサービス更新に進ませない

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = bootstrap::load_config()?;
    let _telemetry = bootstrap::init_telemetry(&config, "migrate")?;
    let aws = bootstrap::aws_config().await;
    let pool = bootstrap::connect_db(&config, &aws).await?;

    payroll_infrastructure::MIGRATOR.run(&pool).await?;
    tracing::info!("migrations applied");
    Ok(())
}
