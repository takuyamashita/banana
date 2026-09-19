use payroll_usecase::ports::repository::RepositoryError;
use platform_db::DbFailure;
use sqlx::migrate::Migrator;

pub use platform_db::connect;

/// infrastructure/migrations を埋め込んだマイグレーター。payroll-migrate から使う
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// sqlx のエラーを usecase の `RepositoryError` に翻訳する。
///
/// `impl From<sqlx::Error> for RepositoryError` は書けない。両方とも infrastructure から見て
/// 外部の型なので、孤児ルール(E0117)に触れる。usecase 側に書けば usecase が sqlx に依存してしまう。
///
/// 再試行すれば通りうるもの(接続できない・ロック待ちのタイムアウト・デッドロック)だけを Unavailable にし、
/// それ以外(列がない・型が合わない・制約違反など、やり直しても直らないもの)は Internal にする
#[allow(clippy::needless_pass_by_value, reason = "map_err(db_err) で渡すため値で受ける")]
pub(crate) fn db_err(err: sqlx::Error) -> RepositoryError {
    match platform_db::classify(&err) {
        // DB のメッセージには値(メールアドレスなど)が入るので、クライアントには返さない
        DbFailure::Conflict => RepositoryError::Conflict("一意制約に違反しました".to_owned()),
        DbFailure::Unavailable => RepositoryError::Unavailable(err.to_string()),
        DbFailure::Internal => RepositoryError::Internal(err.to_string()),
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
