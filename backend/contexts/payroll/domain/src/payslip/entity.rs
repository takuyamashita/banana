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

/// 給与明細がとりうる状態
pub trait State: Copy {
    fn status(self) -> PayslipStatus;
}

impl State for Draft {
    fn status(self) -> PayslipStatus {
        PayslipStatus::Draft
    }
}

impl State for Finalized {
    fn status(self) -> PayslipStatus {
        PayslipStatus::Finalized
    }
}

impl State for PayslipStatus {
    fn status(self) -> PayslipStatus {
        self
    }
}

/// 給与明細には明細行が1件以上ある。稼働のない月の給与明細は作らない
fn ensure_lines(lines: &[PayslipLine]) -> Result<(), PayslipError> {
    if lines.is_empty() {
        return Err(PayslipError::EmptyLines);
    }
    Ok(())
}

/// 給与明細。派遣社員1人の、ある1か月分の給与を表す。
///
/// 同じ派遣社員・同じ月の給与明細は、有効なものが常に1つだけ存在する。
#[derive(Debug)]
pub struct Payslip<S = PayslipStatus, Id = PayslipId> {
    /// 給与明細番号
    id: Id,
    /// 給与を受け取る派遣社員
    staff_id: StaffId,
    /// 対象月
    period: PayPeriod,
    /// 案件ごとの稼働と時給。1件以上ある
    lines: Vec<PayslipLine>,
    /// 作成中か、確定済みか
    state: S,
}

/// 作成中の給与明細
pub type DraftPayslip<Id = PayslipId> = Payslip<Draft, Id>;

/// 確定済みの給与明細
pub type FinalizedPayslip<Id = PayslipId> = Payslip<Finalized, Id>;

/// まだ登録していない給与明細
pub type NewPayslip = Payslip<PayslipStatus, Unsaved>;

impl<S, Id> Payslip<S, Id> {
    fn with_state<T>(self, state: T) -> Payslip<T, Id> {
        let Self { id, staff_id, period, lines, state: _ } = self;
        Payslip { id, staff_id, period, lines, state }
    }
}

impl<S: State, Id> Payslip<S, Id> {
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

    #[must_use]
    pub fn status(&self) -> PayslipStatus {
        self.state.status()
    }
}

impl<S: State> Payslip<S, PayslipId> {
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
        (self.with_state(Finalized), event)
    }
}

/// 状態ごとに分けた給与明細
#[derive(Debug)]
pub enum PayslipState<Id = PayslipId> {
    Draft(DraftPayslip<Id>),
    Finalized(FinalizedPayslip<Id>),
}

impl<Id> Payslip<PayslipStatus, Id> {
    /// 作成中か確定済みかで分ける
    pub fn into_state(self) -> PayslipState<Id> {
        match self.state {
            PayslipStatus::Draft => PayslipState::Draft(self.with_state(Draft)),
            PayslipStatus::Finalized => PayslipState::Finalized(self.with_state(Finalized)),
        }
    }
}

impl Payslip<PayslipStatus, Unsaved> {
    /// 作成中の給与明細を新しく作る。明細行は1件以上必要
    pub fn draft(
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
    ) -> Result<DraftPayslip<Unsaved>, PayslipError> {
        ensure_lines(&lines)?;
        Ok(Payslip { id: Unsaved, staff_id, period, lines, state: Draft })
    }
}

impl Payslip<PayslipStatus, PayslipId> {
    /// 登録済みの給与明細を、記録されている内容から組み立て直す
    pub fn reconstruct(
        id: PayslipId,
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
        status: PayslipStatus,
    ) -> Result<Self, PayslipError> {
        ensure_lines(&lines)?;
        Ok(Payslip { id, staff_id, period, lines, state: status })
    }
}

impl<Id> From<DraftPayslip<Id>> for Payslip<PayslipStatus, Id> {
    fn from(payslip: DraftPayslip<Id>) -> Self {
        payslip.with_state(PayslipStatus::Draft)
    }
}

impl<Id> From<FinalizedPayslip<Id>> for Payslip<PayslipStatus, Id> {
    fn from(payslip: FinalizedPayslip<Id>) -> Self {
        payslip.with_state(PayslipStatus::Finalized)
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
        assert_eq!(finalized.status(), PayslipStatus::Finalized);
    }
}
