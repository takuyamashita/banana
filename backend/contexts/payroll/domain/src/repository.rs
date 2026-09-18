use thiserror::Error;

/// リポジトリ実装が返すエラー。sqlx などの下位エラーは infrastructure でこれに翻訳する
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// 一意制約違反など、既存データと衝突した
    #[error("既存のデータと衝突しました: {0}")]
    Conflict(String),
    /// DB に想定外の値が入っていて、集約を組み立てられない
    #[error("保存されたデータが壊れています: {0}")]
    CorruptedData(String),
    /// 接続断など、呼び出し側では対処できない失敗
    #[error("リポジトリの操作に失敗しました: {0}")]
    Unavailable(String),
}
