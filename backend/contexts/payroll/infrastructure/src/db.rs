use payroll_usecase::ports::repository::RepositoryError;
use sqlx::Executor;
use sqlx::migrate::Migrator;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

/// infrastructure/migrations を埋め込んだマイグレーター。app/migrate から使う
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// 接続ごとに分離レベルとタイムゾーンを固定する。
/// 既定の REPEATABLE READ はギャップロックが広く、存在確認→挿入でデッドロックを招きやすい
pub async fn connect(url: &str, max_connections: u32) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(max_connections)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("set session transaction isolation level read committed").await?;
                conn.execute("set time_zone = '+00:00'").await?;
                Ok(())
            })
        })
        .connect(url)
        .await
}

/// sqlx のエラーを usecase の `RepositoryError` に翻訳する。
///
/// `impl From<sqlx::Error> for RepositoryError` は書けない。両方とも infrastructure から見て
/// 外部の型なので、孤児ルール(E0117)に触れる。usecase 側に書けば usecase が sqlx に依存してしまう
#[allow(clippy::needless_pass_by_value, reason = "map_err(db_err) で渡すため値で受ける")]
pub(crate) fn db_err(err: sqlx::Error) -> RepositoryError {
    if let Some(db) = err.as_database_error()
        && db.is_unique_violation()
    {
        // 制約名や値を含む DB のメッセージはクライアントに返さず、ログにだけ残す
        tracing::info!(detail = db.message(), "unique constraint violated");
        return RepositoryError::Conflict("一意制約に違反しました".to_owned());
    }
    RepositoryError::Unavailable(err.to_string())
}

pub(crate) fn corrupted(err: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::CorruptedData(err.to_string())
}
