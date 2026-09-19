use std::time::Duration;

use payroll_usecase::ports::repository::RepositoryError;
use sqlx::Executor;
use sqlx::migrate::Migrator;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

/// infrastructure/migrations を埋め込んだマイグレーター。app/migrate から使う
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

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

/// sqlx のエラーを usecase の `RepositoryError` に翻訳する。
///
/// `impl From<sqlx::Error> for RepositoryError` は書けない。両方とも infrastructure から見て
/// 外部の型なので、孤児ルール(E0117)に触れる。usecase 側に書けば usecase が sqlx に依存してしまう。
///
/// 再試行すれば通りうるもの(接続できない・ロック待ちのタイムアウト・デッドロック)だけを Unavailable にし、
/// それ以外(列がない・型が合わない・制約違反など、やり直しても直らないもの)は Internal にする
#[allow(clippy::needless_pass_by_value, reason = "map_err(db_err) で渡すため値で受ける")]
pub(crate) fn db_err(err: sqlx::Error) -> RepositoryError {
    match &err {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            // DB のメッセージ(Duplicate entry '<値>' for key '<制約名>')はクライアントに返さない。
            // 値にはメールアドレスなどが入るので、ログにも制約名だけを残す
            let constraint = db.message().rsplit_once("for key ").map_or("-", |(_, key)| key);
            tracing::info!(constraint, "unique constraint violated");
            RepositoryError::Conflict("一意制約に違反しました".to_owned())
        }
        // 1205: ロック待ちのタイムアウト、1213: デッドロック。やり直せば通りうる
        sqlx::Error::Database(db)
            if matches!(
                db.try_downcast_ref().map(sqlx::mysql::MySqlDatabaseError::number),
                Some(1205 | 1213)
            ) =>
        {
            RepositoryError::Unavailable(err.to_string())
        }
        sqlx::Error::Io(_)
        | sqlx::Error::Tls(_)
        | sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::WorkerCrashed => RepositoryError::Unavailable(err.to_string()),
        _ => RepositoryError::Internal(err.to_string()),
    }
}

pub(crate) fn corrupted(err: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::CorruptedData(err.to_string())
}

/// update が記録を1行書き換えたかを確かめる。接続は FOUND_ROWS 付きなので、値が変わらなくても
/// 一致した行は数えられる。0 行なら記録されていないものを更新しようとした(組み立ての誤り)
pub(crate) fn ensure_updated(rows: u64, what: &str, id: i64) -> Result<(), RepositoryError> {
    if rows == 1 {
        return Ok(());
    }
    Err(RepositoryError::Internal(format!("記録されていない{what}は更新できません: {id}")))
}
