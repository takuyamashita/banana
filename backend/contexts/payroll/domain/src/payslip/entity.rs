use std::marker::PhantomData;

use platform_kernel::{Money, Unsaved};

use super::{PayPeriod, PayslipError, PayslipEvent, PayslipId, PayslipLine};
use crate::staff::StaffId;

/// 給与明細の状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayslipStatus {
    /// 作成中。内容を確かめている段階で、まだ支給額として確定していない
    Draft,
    /// 確定済み。支給額が決まり、以後は変更できない。振込の対象になる
    Finalized,
}

/// 作成中。内容を確かめている段階で、まだ支給額として確定していない
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Draft;

/// 確定済み。支給額が決まり、以後は変更できない。振込の対象になる
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Finalized;

/// ある状態にある給与明細。派遣社員1人の、ある1か月分の給与を表す。
///
/// 同じ派遣社員・同じ月の給与明細は、有効なものが常に1つだけ存在する。
#[derive(Debug)]
pub struct PayslipIn<State, Id = PayslipId> {
    /// 給与明細番号
    id: Id,
    /// 給与を受け取る派遣社員
    staff_id: StaffId,
    /// 対象月
    period: PayPeriod,
    /// 案件ごとの稼働と時給。1件以上ある
    lines: Vec<PayslipLine>,
    /// 作成中か、確定済みか
    state: PhantomData<State>,
}

/// 作成中の給与明細
pub type DraftPayslip<Id = PayslipId> = PayslipIn<Draft, Id>;

/// 確定済みの給与明細
pub type FinalizedPayslip<Id = PayslipId> = PayslipIn<Finalized, Id>;

impl<State, Id> PayslipIn<State, Id> {
    fn with_id(
        id: Id,
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
    ) -> Result<Self, PayslipError> {
        if lines.is_empty() {
            return Err(PayslipError::EmptyLines);
        }
        Ok(Self { id, staff_id, period, lines, state: PhantomData })
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

    /// 支給額。各明細行の金額(それぞれ円未満切り捨て済み)の合計
    #[must_use]
    pub fn total(&self) -> Money {
        self.lines.iter().map(PayslipLine::amount).fold(Money::ZERO, |acc, m| acc + m)
    }
}

impl<State> PayslipIn<State, PayslipId> {
    #[must_use]
    pub fn id(&self) -> PayslipId {
        self.id
    }
}

impl<Id> DraftPayslip<Id> {
    /// 給与明細を確定し、支給額を決める。確定した給与明細と、確定したという出来事を返す
    pub fn finalize(self) -> (FinalizedPayslip<Id>, PayslipEvent) {
        let event = PayslipEvent::Finalized {
            staff_id: self.staff_id,
            period: self.period,
            total: self.total(),
        };
        let Self { id, staff_id, period, lines, state: _ } = self;
        (PayslipIn { id, staff_id, period, lines, state: PhantomData }, event)
    }
}

/// 給与明細。作成中か確定済みのどちらか
#[derive(Debug)]
pub enum Payslip<Id = PayslipId> {
    Draft(DraftPayslip<Id>),
    Finalized(FinalizedPayslip<Id>),
}

/// まだ登録していない給与明細
pub type NewPayslip = Payslip<Unsaved>;

macro_rules! each_state {
    ($payslip:expr, $p:ident => $body:expr) => {
        match $payslip {
            Payslip::Draft($p) => $body,
            Payslip::Finalized($p) => $body,
        }
    };
}

impl<Id> Payslip<Id> {
    #[must_use]
    pub fn staff_id(&self) -> StaffId {
        each_state!(self, p => p.staff_id())
    }

    #[must_use]
    pub fn period(&self) -> PayPeriod {
        each_state!(self, p => p.period())
    }

    #[must_use]
    pub fn lines(&self) -> &[PayslipLine] {
        each_state!(self, p => p.lines())
    }

    /// 支給額。各明細行の金額(それぞれ円未満切り捨て済み)の合計
    #[must_use]
    pub fn total(&self) -> Money {
        each_state!(self, p => p.total())
    }

    #[must_use]
    pub fn status(&self) -> PayslipStatus {
        match self {
            Self::Draft(_) => PayslipStatus::Draft,
            Self::Finalized(_) => PayslipStatus::Finalized,
        }
    }
}

impl Payslip<Unsaved> {
    /// 作成中の給与明細を新しく作る。明細行は1件以上必要
    pub fn draft(
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
    ) -> Result<DraftPayslip<Unsaved>, PayslipError> {
        PayslipIn::with_id(Unsaved, staff_id, period, lines)
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
        Ok(match status {
            PayslipStatus::Draft => Self::Draft(PayslipIn::with_id(id, staff_id, period, lines)?),
            PayslipStatus::Finalized => {
                Self::Finalized(PayslipIn::with_id(id, staff_id, period, lines)?)
            }
        })
    }

    #[must_use]
    pub fn id(&self) -> PayslipId {
        each_state!(self, p => p.id())
    }
}

impl<Id> From<DraftPayslip<Id>> for Payslip<Id> {
    fn from(payslip: DraftPayslip<Id>) -> Self {
        Self::Draft(payslip)
    }
}

impl<Id> From<FinalizedPayslip<Id>> for Payslip<Id> {
    fn from(payslip: FinalizedPayslip<Id>) -> Self {
        Self::Finalized(payslip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payslip::WorkMinutes;
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
    fn finalize_returns_the_finalized_payslip_and_the_event() {
        let period = PayPeriod::new(2026, 9).unwrap();
        let draft = NewPayslip::draft(staff(), period, vec![line(600, 1_500)]).unwrap();

        let (finalized, event) = draft.finalize();

        assert_eq!(
            event,
            PayslipEvent::Finalized {
                staff_id: staff(),
                period,
                total: Money::from_yen(15_000).unwrap()
            }
        );
        assert_eq!(Payslip::from(finalized).status(), PayslipStatus::Finalized);
    }
}
