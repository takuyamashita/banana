//! MySQL への接続と、DB のエラーの分類。
//!
//! どのサービスも同じ決まりで接続する(分離レベル・タイムゾーン・ロック待ちの上限)。
//! エラーは「やり直せば通りうるか」で分け、各コンテキストが自分のエラー型に翻訳する

use std::time::Duration;

use sqlx::Executor;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

/// プールから接続を借りるのを待つ上限。混んでいるときに、リクエストが長く待ち続けないようにする
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// 接続ごとに分離レベル・タイムゾーン・ロック待ちの上限を固定する。
/// 既定の REPEATABLE READ はギャップロックが広く、存在確認→挿入でデッドロックを招きやすい。
/// ロック待ちの既定(50秒)は長すぎるので、10秒で打ち切る(打ち切られたら Unavailable になる)
pub async fn connect(url: &str, max_connections: u32) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("set session transaction isolation level read committed").await?;
                conn.execute("set time_zone = '+00:00'").await?;
                conn.execute("set session innodb_lock_wait_timeout = 10").await?;
                Ok(())
            })
        })
        .connect(url)
        .await
}

/// DB のエラーの分け方
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbFailure {
    /// 一意制約に違反した(同じものが既にある)
    Conflict,
    /// やり直せば通りうる(接続できない・ロック待ちのタイムアウト・デッドロック)
    Unavailable,
    /// やり直しても直らない(列がない・型が合わない・制約違反など)
    Internal,
}

/// DB のエラーを分ける。一意制約の違反は、制約名だけをログに残す
/// (DB のメッセージ `Duplicate entry '<値>' for key '<制約名>'` の値にはメールアドレスなどが入る)
pub fn classify(err: &sqlx::Error) -> DbFailure {
    match err {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            let constraint = db.message().rsplit_once("for key ").map_or("-", |(_, key)| key);
            tracing::info!(constraint, "unique constraint violated");
            DbFailure::Conflict
        }
        // 1205: ロック待ちのタイムアウト、1213: デッドロック
        sqlx::Error::Database(db)
            if matches!(
                db.try_downcast_ref().map(sqlx::mysql::MySqlDatabaseError::number),
                Some(1205 | 1213)
            ) =>
        {
            DbFailure::Unavailable
        }
        sqlx::Error::Io(_)
        | sqlx::Error::Tls(_)
        | sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::WorkerCrashed => DbFailure::Unavailable,
        _ => DbFailure::Internal,
    }
}
