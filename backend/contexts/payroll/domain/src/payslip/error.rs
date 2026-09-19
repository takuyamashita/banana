use thiserror::Error;

/// 給与明細の業務ルールに反したときの理由
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PayslipError {
    /// 明細行が1件もない。稼働のない月の給与明細は作らない
    #[error("給与明細には最低1件の行が必要です")]
    EmptyLines,
    /// 明細行が多すぎる。1か月に1人が従事する案件の数としてありえない
    #[error("明細行は{max}件までです")]
    TooManyLines { max: usize },
    /// 稼働時間が15分単位でない・0分、または1か月の上限を超えている
    #[error("稼働時間は15分単位で、1か月744時間までです")]
    InvalidWorkMinutes,
    /// 時給が1円未満か、上限を超えている
    #[error("時給は1円以上{max}円以下です")]
    InvalidHourlyRate { max: u32 },
    /// 年が業務で扱う範囲にない、または月が1〜12の範囲にない
    #[error("対象年月が不正です")]
    InvalidPeriod,
}
