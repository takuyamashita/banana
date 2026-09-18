use thiserror::Error;

/// 給与明細の業務ルールに反したときの理由
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PayslipError {
    /// 明細行が1件もない。稼働のない月の給与明細は作らない
    #[error("給与明細には最低1件の行が必要です")]
    EmptyLines,
    /// 稼働時間が15分に満たない、または1か月の上限を超えている
    #[error("稼働時間が不正です")]
    InvalidWorkMinutes,
    /// 月が1〜12の範囲にない
    #[error("対象年月が不正です")]
    InvalidPeriod,
    /// 給与明細番号が正の数でない
    #[error("給与明細IDが不正です")]
    InvalidId,
    /// 確定済みの給与明細をもう一度確定しようとした
    #[error("確定済みの給与明細は変更できません")]
    AlreadyFinalized,
}
