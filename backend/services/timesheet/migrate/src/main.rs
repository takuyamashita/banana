//! 勤怠(timesheet)サービスのマイグレーション。ECS の単発タスクとしてデプロイ前に動かす。
//! 失敗したら非0で終了し、デプロイをサービス更新に進ませない

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = timesheet_bootstrap::load_config()?;
    let _telemetry = timesheet_bootstrap::init_telemetry(&config, "timesheet-migrate")?;
    timesheet_bootstrap::migrate(&config).await
}
