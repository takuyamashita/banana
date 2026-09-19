use platform_db::DbFailure;
use sqlx::migrate::Migrator;
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};
use timesheet_usecase::ports::repository::RepositoryError;

pub use platform_db::connect;

/// infrastructure/migrations を埋め込んだマイグレーター。timesheet-migrate から使う
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// sqlx のエラーを usecase の `RepositoryError` に翻訳する(やり直せば通りうるものだけ Unavailable)
#[allow(clippy::needless_pass_by_value, reason = "map_err(db_err) で渡すため値で受ける")]
pub(crate) fn db_err(err: sqlx::Error) -> RepositoryError {
    match platform_db::classify(&err) {
        DbFailure::Conflict => RepositoryError::Conflict("一意制約に違反しました".to_owned()),
        DbFailure::Unavailable => RepositoryError::Unavailable(err.to_string()),
        DbFailure::Internal => RepositoryError::Internal(err.to_string()),
    }
}

pub(crate) fn corrupted(err: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::CorruptedData(err.to_string())
}

/// update が記録を1行書き換えたかを確かめる。0 行なら記録されていないものを更新しようとした(組み立ての誤り)
pub(crate) fn ensure_updated(rows: u64, what: &str, id: i64) -> Result<(), RepositoryError> {
    if rows == 1 {
        return Ok(());
    }
    Err(RepositoryError::Internal(format!("記録されていない{what}は更新できません: {id}")))
}

/// DB の日時(UTC で保存している)を、UTC の日時にする
pub(crate) fn utc(at: PrimitiveDateTime) -> OffsetDateTime {
    at.assume_utc()
}

/// UTC の日時を、DB に保存する形にする
pub(crate) fn to_db(at: OffsetDateTime) -> PrimitiveDateTime {
    let at = at.to_offset(UtcOffset::UTC);
    PrimitiveDateTime::new(at.date(), at.time())
}
