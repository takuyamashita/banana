//! 給与(payroll)サービスのマイグレーション。ECS の単発タスクとしてデプロイ前に動かす。
//! 失敗したら非0で終了し、デプロイをサービス更新に進ませない

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = payroll_bootstrap::load_config()?;
    let _telemetry = payroll_bootstrap::init_telemetry(&config, "payroll-migrate")?;
    payroll_bootstrap::migrate(&config).await
}
