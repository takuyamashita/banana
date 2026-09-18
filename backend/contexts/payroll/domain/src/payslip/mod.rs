//! 給与明細(payslip)。
//!
//! 派遣社員1人の、ある1か月分の給与を表す。案件ごとの稼働時間と時給を明細行として持ち、
//! その合計が支給額になる。管理者が内容を確かめて「確定」すると、以後は変更できず、
//! 確定した事実を振込などの後続業務に知らせる。

use platform_kernel::{Money, Unsaved};
use thiserror::Error;
use time::{Date, Month, OffsetDateTime, UtcOffset};

use crate::project::ProjectId;
use crate::staff::StaffId;

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

/// 給与明細番号。登録された給与明細を一意に指す正の整数
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PayslipId(i64);

impl PayslipId {
    pub fn from_i64(value: i64) -> Result<Self, PayslipError> {
        if value <= 0 {
            return Err(PayslipError::InvalidId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

/// 給与の対象月(「2026年9月分」)。
///
/// 月の区切りは日本時間で数える。9月分は 9月1日 0:00(JST)から 10月1日 0:00(JST)の直前までに
/// 行われた稼働が対象になる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PayPeriod {
    /// 西暦年
    year: u16,
    /// 月(1〜12)
    month: u8,
}

impl PayPeriod {
    pub fn new(year: u16, month: u8) -> Result<Self, PayslipError> {
        if !(1..=12).contains(&month) {
            return Err(PayslipError::InvalidPeriod);
        }
        Ok(Self { year, month })
    }

    #[must_use]
    pub fn year(&self) -> u16 {
        self.year
    }

    #[must_use]
    pub fn month(&self) -> u8 {
        self.month
    }

    /// この月に含まれる時刻の範囲を UTC で返す。
    ///
    /// 始まり(月初 0:00 JST)を含み、終わり(翌月初 0:00 JST)を含まない。
    /// 「月末 23:59:59 まで」とすると、その1秒の間の時刻が漏れるため、終わりは翌月初で表す
    #[must_use]
    pub fn range_utc(&self) -> (OffsetDateTime, OffsetDateTime) {
        let start = self.first_moment_jst();
        let end = self.next().first_moment_jst();
        (start.to_offset(UtcOffset::UTC), end.to_offset(UtcOffset::UTC))
    }

    /// 月初 0:00(JST)。日本にはサマータイムがないので、常に UTC+9 で数える
    fn first_moment_jst(self) -> OffsetDateTime {
        let jst = UtcOffset::from_hms(9, 0, 0).expect("JST");
        Date::from_calendar_date(
            i32::from(self.year),
            Month::try_from(self.month).expect("1..=12"),
            1,
        )
        .expect("valid date")
        .midnight()
        .assume_offset(jst)
    }

    fn next(self) -> Self {
        if self.month == 12 {
            Self { year: self.year + 1, month: 1 }
        } else {
            Self { year: self.year, month: self.month + 1 }
        }
    }
}

/// 1つの案件での、1か月分の稼働時間(分)。
///
/// 稼働は15分単位で数え、15分に満たない端数は切り捨てる(100分の稼働は90分として扱う)。
/// 切り捨てた結果が0分になるもの(14分以下)は稼働として認めない。
/// 1か月の上限は 744時間(31日 × 24時間)で、これを超える稼働はありえないので受け付けない
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkMinutes(u32);

/// 1か月の稼働時間の上限(分)。31日 × 24時間
const MAX_WORK_MINUTES: u32 = 744 * 60;

impl WorkMinutes {
    pub fn from_minutes(minutes: u32) -> Result<Self, PayslipError> {
        let floored = minutes - minutes % 15;
        if floored == 0 || minutes > MAX_WORK_MINUTES {
            return Err(PayslipError::InvalidWorkMinutes);
        }
        Ok(Self(floored))
    }

    /// 15分単位に切り捨てた後の分数
    #[must_use]
    pub fn as_minutes(&self) -> u32 {
        self.0
    }
}

/// 給与明細の1行。どの案件で、どれだけ働き、時給はいくらだったか
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayslipLine {
    /// 稼働した案件
    project_id: ProjectId,
    /// その案件での、この月の稼働時間
    work_minutes: WorkMinutes,
    /// その案件での時給(円)。同じ派遣社員でも案件ごとに異なる
    hourly_rate: Money,
}

impl PayslipLine {
    #[must_use]
    pub fn new(project_id: ProjectId, work_minutes: WorkMinutes, hourly_rate: Money) -> Self {
        Self { project_id, work_minutes, hourly_rate }
    }

    #[must_use]
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }

    #[must_use]
    pub fn work_minutes(&self) -> WorkMinutes {
        self.work_minutes
    }

    #[must_use]
    pub fn hourly_rate(&self) -> Money {
        self.hourly_rate
    }

    /// この行の支給額。時給 × 稼働分 ÷ 60 で、円未満は切り捨てる
    /// (1,001円で15分なら 250.25円 → 250円)
    #[must_use]
    pub fn amount(&self) -> Money {
        (self.hourly_rate * self.work_minutes.as_minutes()).div_floor(60)
    }
}

/// 給与明細の状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayslipStatus {
    /// 作成中。内容を確かめている段階で、まだ支給額として確定していない
    Draft,
    /// 確定済み。支給額が決まり、以後は変更できない。振込の対象になる
    Finalized,
}

/// 給与明細に起きた、他の業務が知るべき出来事
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayslipEvent {
    /// 給与明細が確定し、支給額が決まった。振込はこれを受けて行う
    Finalized {
        /// 支給を受ける派遣社員
        staff_id: StaffId,
        /// 対象月
        period: PayPeriod,
        /// 確定した支給額(全明細行の合計)
        total: Money,
    },
}

/// 給与明細。派遣社員1人の、ある1か月分の給与を表す。
///
/// 同じ派遣社員・同じ月の給与明細は、有効なものが常に1つだけ存在する。
/// `Id` はまだ登録していない給与明細なら [`Unsaved`]、登録済みなら [`PayslipId`]
#[derive(Debug)]
pub struct Payslip<Id = PayslipId> {
    /// 給与明細番号
    id: Id,
    /// 給与を受け取る派遣社員
    staff_id: StaffId,
    /// 対象月
    period: PayPeriod,
    /// 案件ごとの稼働と時給。1件以上ある
    lines: Vec<PayslipLine>,
    /// 作成中か、確定済みか
    status: PayslipStatus,
    /// まだ他の業務に知らせていない出来事
    events: Vec<PayslipEvent>,
}

/// まだ登録していない給与明細
pub type NewPayslip = Payslip<Unsaved>;

impl<Id> Payslip<Id> {
    fn with_id(
        id: Id,
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
        status: PayslipStatus,
    ) -> Result<Self, PayslipError> {
        if lines.is_empty() {
            return Err(PayslipError::EmptyLines);
        }
        Ok(Self { id, staff_id, period, lines, status, events: Vec::new() })
    }

    #[must_use]
    pub fn staff_id(&self) -> StaffId {
        self.staff_id
    }

    #[must_use]
    pub fn period(&self) -> PayPeriod {
        self.period
    }

    #[must_use]
    pub fn lines(&self) -> &[PayslipLine] {
        &self.lines
    }

    #[must_use]
    pub fn status(&self) -> PayslipStatus {
        self.status
    }

    /// 支給額。各明細行の金額(それぞれ円未満切り捨て済み)の合計
    #[must_use]
    pub fn total(&self) -> Money {
        self.lines.iter().map(PayslipLine::amount).fold(Money::ZERO, |acc, m| acc + m)
    }

    /// 給与明細を確定し、支給額を決める。確定できるのは作成中のものだけで、
    /// 確定すると「確定した」という出来事を記録する
    pub fn finalize(&mut self) -> Result<(), PayslipError> {
        if self.status != PayslipStatus::Draft {
            return Err(PayslipError::AlreadyFinalized);
        }
        self.status = PayslipStatus::Finalized;

        let event = PayslipEvent::Finalized {
            staff_id: self.staff_id,
            period: self.period,
            total: self.total(),
        };
        self.events.push(event);
        Ok(())
    }

    /// まだ知らせていない出来事を取り出す。取り出した出来事は給与明細から消える
    pub fn take_events(&mut self) -> Vec<PayslipEvent> {
        std::mem::take(&mut self.events)
    }
}

impl Payslip<Unsaved> {
    /// 作成中の給与明細を新しく作る。明細行は1件以上必要
    pub fn draft(
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
    ) -> Result<Self, PayslipError> {
        Self::with_id(Unsaved, staff_id, period, lines, PayslipStatus::Draft)
    }
}

impl Payslip<PayslipId> {
    /// 登録済みの給与明細を、記録されている内容から組み立て直す
    pub fn reconstruct(
        id: PayslipId,
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
        status: PayslipStatus,
    ) -> Result<Self, PayslipError> {
        Self::with_id(id, staff_id, period, lines, status)
    }

    #[must_use]
    pub fn id(&self) -> PayslipId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(minutes: u32, rate: i64) -> PayslipLine {
        PayslipLine::new(
            ProjectId::from_i64(1).unwrap(),
            WorkMinutes::from_minutes(minutes).unwrap(),
            Money::from_yen(rate).unwrap(),
        )
    }

    fn staff() -> StaffId {
        StaffId::from_i64(1).unwrap()
    }

    #[test]
    fn work_minutes_are_floored_to_15() {
        assert_eq!(WorkMinutes::from_minutes(100).unwrap().as_minutes(), 90);
    }

    #[test]
    fn work_minutes_that_floor_to_zero_are_rejected() {
        assert_eq!(WorkMinutes::from_minutes(14), Err(PayslipError::InvalidWorkMinutes));
        assert_eq!(WorkMinutes::from_minutes(0), Err(PayslipError::InvalidWorkMinutes));
    }

    #[test]
    fn work_minutes_over_a_month_are_rejected() {
        assert_eq!(WorkMinutes::from_minutes(744 * 60 + 1), Err(PayslipError::InvalidWorkMinutes));
    }

    #[test]
    fn amount_truncates_below_one_yen() {
        // 1,001円 × 15分 / 60 = 250.25円 → 250円
        assert_eq!(line(15, 1_001).amount().as_yen(), 250);
    }

    #[test]
    fn empty_lines_are_rejected() {
        let period = PayPeriod::new(2026, 9).unwrap();
        assert_eq!(
            NewPayslip::draft(staff(), period, vec![]).unwrap_err(),
            PayslipError::EmptyLines
        );
    }

    #[test]
    fn finalize_records_event_once() {
        let period = PayPeriod::new(2026, 9).unwrap();
        let mut p = NewPayslip::draft(staff(), period, vec![line(600, 1_500)]).unwrap();

        p.finalize().unwrap();
        assert_eq!(p.finalize(), Err(PayslipError::AlreadyFinalized));

        let events = p.take_events();
        assert_eq!(
            events,
            vec![PayslipEvent::Finalized {
                staff_id: staff(),
                period,
                total: Money::from_yen(15_000).unwrap()
            }]
        );
        assert!(p.take_events().is_empty());
    }

    #[test]
    fn period_range_is_half_open_in_jst() {
        let (start, end) = PayPeriod::new(2026, 12).unwrap().range_utc();
        assert_eq!(start.to_string(), "2026-11-30 15:00:00.0 +00:00:00");
        assert_eq!(end.to_string(), "2026-12-31 15:00:00.0 +00:00:00");
    }

    #[test]
    fn month_13_is_rejected() {
        assert_eq!(PayPeriod::new(2026, 13), Err(PayslipError::InvalidPeriod));
    }
}
