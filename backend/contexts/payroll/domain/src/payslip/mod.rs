//! 給与明細(payslip)集約。丸めルールと確定後の変更禁止がこのドメインの中核

use async_trait::async_trait;
use platform_kernel::Money;
use thiserror::Error;
use time::{Date, Month, OffsetDateTime, UtcOffset};

use crate::Unsaved;
use crate::project::ProjectId;
use crate::repository::RepositoryError;
use crate::staff::StaffId;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PayslipError {
    #[error("給与明細には最低1件の行が必要です")]
    EmptyLines,
    #[error("稼働時間が不正です")]
    InvalidWorkMinutes,
    #[error("対象年月が不正です")]
    InvalidPeriod,
    #[error("給与明細IDが不正です")]
    InvalidId,
    #[error("確定済みの給与明細は変更できません")]
    AlreadyFinalized,
}

// DBが採番するBIGINT。アプリ側では生成しないので generate() は持たない
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PayPeriod {
    year: u16,
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

    // 「2026年9月分」がUTCのどの区間かをdomainが答える。
    // [月初00:00 JST, 翌月初00:00 JST) の半開区間。末尾を 23:59:59 にすると
    // マイクロ秒の端が漏れるので半開にする
    #[must_use]
    pub fn range_utc(&self) -> (OffsetDateTime, OffsetDateTime) {
        let start = self.first_moment_jst();
        let end = self.next().first_moment_jst();
        (start.to_offset(UtcOffset::UTC), end.to_offset(UtcOffset::UTC))
    }

    // JSTにサマータイムがないので、固定オフセットで足りる
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

// 稼働時間。生成時に15分単位へ切り捨てる
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkMinutes(u32);

const MAX_WORK_MINUTES: u32 = 744 * 60;

impl WorkMinutes {
    pub fn from_minutes(minutes: u32) -> Result<Self, PayslipError> {
        let floored = minutes - minutes % 15;
        // 切り捨て後に0分になる値(1〜14分)も弾く
        if floored == 0 || minutes > MAX_WORK_MINUTES {
            return Err(PayslipError::InvalidWorkMinutes);
        }
        Ok(Self(floored))
    }

    #[must_use]
    pub fn as_minutes(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayslipLine {
    project_id: ProjectId,
    work_minutes: WorkMinutes,
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

    // 時給 × 稼働分 / 60。円未満切り捨てという選択が業務ルール
    #[must_use]
    pub fn amount(&self) -> Money {
        (self.hourly_rate * self.work_minutes.as_minutes()).div_floor(60)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayslipStatus {
    Draft,
    Finalized,
}

// 集約が発行するドメインイベント。serdeのderiveは付けない。
// 新規時はIDが未採番なので、IDはoutbox行の aggregate_id に入れる(infrastructureの責務)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayslipEvent {
    Finalized { staff_id: StaffId, period: PayPeriod, total: Money },
}

/// 給与明細。`Id` は保存済みなら `PayslipId`、未保存なら `Unsaved`。
/// 不変条件・状態遷移・合計の業務ルールは `impl<Id>` に1回だけ書き、未保存・保存済みの両方で使う
#[derive(Debug)]
pub struct Payslip<Id = PayslipId> {
    id: Id,
    staff_id: StaffId,
    period: PayPeriod,
    lines: Vec<PayslipLine>,
    status: PayslipStatus,
    events: Vec<PayslipEvent>,
}

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

    #[must_use]
    pub fn total(&self) -> Money {
        self.lines.iter().map(PayslipLine::amount).fold(Money::ZERO, |acc, m| acc + m)
    }

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

    pub fn take_events(&mut self) -> Vec<PayslipEvent> {
        std::mem::take(&mut self.events)
    }
}

impl Payslip<Unsaved> {
    pub fn draft(
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
    ) -> Result<Self, PayslipError> {
        Self::with_id(Unsaved, staff_id, period, lines, PayslipStatus::Draft)
    }
}

impl Payslip<PayslipId> {
    // 永続化からの再構築専用。repository実装からのみ呼ぶ(usecase・handler は clippy で禁止)
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

// 採番があるため insert と update を分ける。insert は採番済みの Payslip を返す
#[async_trait]
pub trait PayslipRepository: Send + Sync {
    async fn insert(&self, new: &mut NewPayslip) -> Result<Payslip, RepositoryError>;
    async fn update(&self, payslip: &mut Payslip) -> Result<(), RepositoryError>;
    async fn find(&self, id: PayslipId) -> Result<Option<Payslip>, RepositoryError>;
    async fn list_by_staff(&self, staff_id: StaffId) -> Result<Vec<Payslip>, RepositoryError>;
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
