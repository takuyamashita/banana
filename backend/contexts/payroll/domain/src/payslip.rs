//! 給与明細(payslip)。
//!
//! 派遣社員1人の、ある1か月分の給与を表す。案件ごとの稼働時間と時給を明細行として持ち、
//! その合計が支給額になる。管理者が内容を確かめて「確定」すると、以後は変更できず、
//! 確定した事実を振込などの後続業務に知らせる。

mod error;
mod id;
mod line;
mod pay_period;
mod work_minutes;

use platform_kernel::{Money, Unsaved};

pub use self::error::PayslipError;
pub use self::id::PayslipId;
pub use self::line::PayslipLine;
pub use self::pay_period::PayPeriod;
pub use self::work_minutes::WorkMinutes;
use crate::staff::StaffId;

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
    use crate::project::ProjectId;

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
}
