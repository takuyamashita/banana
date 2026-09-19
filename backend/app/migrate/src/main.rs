//! 適用済みマイグレーションは _sqlx_migrations に記録されるので、再実行しても安全。
//! 失敗したら非0で終了し、デプロイをサービス更新に進ませない

use std::str::FromStr;

use anyhow::Context;
use migrate::{Credentials, ensure_app_user};
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = bootstrap::load_config()?;
    let _telemetry = bootstrap::init_telemetry(&config, "migrate")?;
    let aws = bootstrap::aws_config().await;
    let secrets = &config.secrets;

    // ローカル: アプリと同じ接続でマイグレーションだけを流す
    if secrets.database_admin_secret_id.is_empty() {
        let pool = bootstrap::connect_db(&config, &aws).await?;
        payroll_infrastructure::MIGRATOR.run(&pool).await?;
        tracing::info!("migrations applied");
        return Ok(());
    }

    // AWS: 接続先はアプリの接続文字列から取り、ユーザーだけ管理者に替える
    let app_url = bootstrap::database_url(&config, &aws).await?;
    let target = MySqlConnectOptions::from_str(&app_url).context("invalid database url")?;
    let database = target.get_database().context("database url has no database name")?.to_owned();
    let admin = Credentials::from_json(
        &bootstrap::read_secret(&aws, &secrets.database_admin_secret_id, "database admin").await?,
    )?;
    let app_user = Credentials::from_json(
        &bootstrap::read_secret(&aws, &secrets.database_app_user_secret_id, "database app user")
            .await?,
    )?;
    let pool = MySqlPoolOptions::new()
        .max_connections(1)
        .connect_with(target.username(&admin.username).password(&admin.password))
        .await
        .context("failed to connect to database as admin")?;

    payroll_infrastructure::MIGRATOR.run(&pool).await?;
    tracing::info!("migrations applied");
    ensure_app_user(&pool, &database, &app_user).await?;
    tracing::info!(user = app_user.username, "application user is ready");
    Ok(())
}
