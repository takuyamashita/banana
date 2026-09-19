use thiserror::Error;

/// 勤務表が業務ルールに合わない理由
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TimesheetError {
    #[error("対象月は 2000年1月〜2999年12月で指定してください")]
    InvalidMonth,
    #[error("稼働は15分単位で、1日24時間までで入力してください")]
    InvalidWorkMinutes,
    #[error("{date} はこの勤務表の月の日ではありません")]
    OutsideMonth { date: String },
    #[error("{date} の同じ案件の稼働が2行あります。1行にまとめてください")]
    DuplicateEntry { date: String },
    #[error("{date} の稼働の合計が24時間を超えています")]
    DayOverflow { date: String },
    #[error("稼働の行は{max}件までです")]
    TooManyEntries { max: usize },
    #[error("稼働が1件もない勤務表は申告できません")]
    NothingToSubmit,
    #[error("差し戻しの理由は1〜500文字で入力してください")]
    InvalidReturnReason,
}
