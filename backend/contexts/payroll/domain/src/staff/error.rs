use thiserror::Error;

/// 派遣社員の業務ルールに反したときの理由
#[derive(Debug, Error, PartialEq, Eq)]
pub enum StaffError {
    /// 表示名が空、または50文字を超えている
    #[error("表示名は1〜50文字で指定してください")]
    InvalidDisplayName,
}
