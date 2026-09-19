use platform_kernel::{Money, Unsaved};
use time::OffsetDateTime;

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

/// 給与明細の内容。作成中でも確定済みでも持つ情報
#[derive(Debug)]
pub struct PayslipContent<Id = PayslipId> {
    /// 給与明細番号
    id: Id,
    /// 給与を受け取る派遣社員
    staff_id: StaffId,
    /// 対象月
    period: PayPeriod,
    /// 案件ごとの稼働と時給。1件以上ある
    lines: Vec<PayslipLine>,
}

impl<Id> PayslipContent<Id> {
    /// 明細行は1件以上必要。稼働のない月の給与明細は作らない
    fn new(
        id: Id,
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
    ) -> Result<Self, PayslipError> {
        if lines.is_empty() {
            return Err(PayslipError::EmptyLines);
        }
        Ok(Self { id, staff_id, period, lines })
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

impl PayslipContent<PayslipId> {
    #[must_use]
    pub fn id(&self) -> PayslipId {
        self.id
    }
}

/// 作成中の給与明細。内容を確かめている段階で、まだ支給額として確定していない
#[derive(Debug)]
pub struct DraftPayslip<Id = PayslipId> {
    content: PayslipContent<Id>,
}

impl<Id> DraftPayslip<Id> {
    #[must_use]
    pub fn content(&self) -> &PayslipContent<Id> {
        &self.content
    }

    /// 給与明細を `finalized_at` の日時で確定し、支給額を決める。
    /// 確定した給与明細と、確定したという出来事を返す
    pub fn finalize(self, finalized_at: OffsetDateTime) -> (FinalizedPayslip<Id>, PayslipEvent) {
        let event = PayslipEvent::Finalized {
            staff_id: self.content.staff_id,
            period: self.content.period,
            total: self.content.total(),
        };
        (FinalizedPayslip { content: self.content, finalized_at }, event)
    }
}

/// 確定済みの給与明細。支給額が決まり、以後は変更できない。振込の対象になる
#[derive(Debug)]
pub struct FinalizedPayslip<Id = PayslipId> {
    content: PayslipContent<Id>,
    /// 確定した日時。この時点で支給額が決まった
    finalized_at: OffsetDateTime,
}

impl<Id> FinalizedPayslip<Id> {
    #[must_use]
    pub fn content(&self) -> &PayslipContent<Id> {
        &self.content
    }

    #[must_use]
    pub fn finalized_at(&self) -> OffsetDateTime {
        self.finalized_at
    }
}

/// 給与明細。派遣社員1人の、ある1か月分の給与を表し、作成中か確定済みのどちらか。
///
/// 同じ派遣社員・同じ月の給与明細は、有効なものが常に1つだけ存在する。
#[derive(Debug)]
pub enum Payslip<Id = PayslipId> {
    Draft(DraftPayslip<Id>),
    Finalized(FinalizedPayslip<Id>),
}

/// まだ登録していない給与明細
pub type NewPayslip = Payslip<Unsaved>;

impl<Id> Payslip<Id> {
    #[must_use]
    pub fn content(&self) -> &PayslipContent<Id> {
        match self {
            Self::Draft(p) => p.content(),
            Self::Finalized(p) => p.content(),
        }
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
        Ok(DraftPayslip { content: PayslipContent::new(Unsaved, staff_id, period, lines)? })
    }
}

impl Payslip<PayslipId> {
    /// 登録済みの作成中の給与明細を、記録されている内容から組み立て直す
    pub fn reconstruct_draft(
        id: PayslipId,
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
    ) -> Result<Self, PayslipError> {
        let content = PayslipContent::new(id, staff_id, period, lines)?;
        Ok(Self::Draft(DraftPayslip { content }))
    }

    /// 登録済みの確定済みの給与明細を、記録されている内容から組み立て直す
    pub fn reconstruct_finalized(
        id: PayslipId,
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
        finalized_at: OffsetDateTime,
    ) -> Result<Self, PayslipError> {
        let content = PayslipContent::new(id, staff_id, period, lines)?;
        Ok(Self::Finalized(FinalizedPayslip { content, finalized_at }))
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
    use time::macros::datetime;

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
        assert_eq!(Payslip::draft(staff(), period, vec![]).unwrap_err(), PayslipError::EmptyLines);
    }

    #[test]
    fn finalize_returns_the_finalized_payslip_and_the_event() {
        let period = PayPeriod::new(2026, 9).unwrap();
        let draft = Payslip::draft(staff(), period, vec![line(600, 1_500)]).unwrap();

        let at = datetime!(2026-09-30 10:00 UTC);

        let (finalized, event) = draft.finalize(at);

        assert_eq!(
            event,
            PayslipEvent::Finalized {
                staff_id: staff(),
                period,
                total: Money::from_yen(15_000).unwrap()
            }
        );
        assert_eq!(finalized.finalized_at(), at);
        assert_eq!(Payslip::from(finalized).status(), PayslipStatus::Finalized);
    }
}
