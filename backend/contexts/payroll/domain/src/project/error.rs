use thiserror::Error;

/// 案件の業務ルールに反したときの理由
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProjectError {
    /// 案件名が空、または100文字を超えている
    #[error("案件名は1〜100文字で指定してください")]
    InvalidName,
}
