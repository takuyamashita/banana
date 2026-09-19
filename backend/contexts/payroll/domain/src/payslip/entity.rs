use platform_kernel::{Money, Unsaved};
use time::OffsetDateTime;

use super::work_minutes::MAX_WORK_MINUTES;
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

/// 1つの給与明細の明細行の上限。1か月に1人が従事する案件の数として十分な数
const MAX_LINES: usize = 100;

/// 給与明細の内容。作成中でも確定済みでも持つ情報
#[derive(Debug)]
pub struct PayslipContent<Id = PayslipId> {
    /// 給与明細番号
    id: Id,
    /// 給与を受け取る派遣社員
    staff_id: StaffId,
    /// 対象月
    period: PayPeriod,
    /// 案件ごとの稼働と時給。1件以上、100件まで
    lines: Vec<PayslipLine>,
    /// 支給額。各明細行の金額(それぞれ円未満切り捨て済み)の合計
    total: Money,
}

impl<Id> PayslipContent<Id> {
    /// 明細行は1件以上必要(稼働のない月の給与明細は作らない)。
    /// 1か月の稼働の合計は、案件をまたいでも 744時間を超えない
    fn new(
        id: Id,
        staff_id: StaffId,
        period: PayPeriod,
        lines: Vec<PayslipLine>,
    ) -> Result<Self, PayslipError> {
        if lines.is_empty() {
            return Err(PayslipError::EmptyLines);
        }
        if lines.len() > MAX_LINES {
            return Err(PayslipError::TooManyLines { max: MAX_LINES });
        }
        let minutes: u32 = lines.iter().map(|l| l.work_minutes().as_minutes()).sum();
        if minutes > MAX_WORK_MINUTES {
            return Err(PayslipError::InvalidWorkMinutes);
        }
        // 明細行の数と1行の金額に上限があるので、合計があふれることはない
        let total = lines
            .iter()
            .try_fold(Money::ZERO, |acc, l| acc.checked_add(l.amount()))
            .ok_or(PayslipError::TooManyLines { max: MAX_LINES })?;
        Ok(Self { id, staff_id, period, lines, total })
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
        self.total
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
}

impl DraftPayslip<PayslipId> {
    /// 給与明細を `finalized_at` の日時で確定し、支給額を決める。
    /// 確定した給与明細と、確定したという出来事を返す。確定できるのは登録済みの給与明細だけ
    pub fn finalize(self, finalized_at: OffsetDateTime) -> (FinalizedPayslip, PayslipEvent) {
        let event = PayslipEvent::Finalized {
            payslip_id: self.content.id,
            staff_id: self.content.staff_id,
            period: self.content.period,
            total: self.content.total,
            finalized_at,
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

/// まだ登録していない給与明細。登録するまでは作成中で、確定もできない
pub type NewPayslip = DraftPayslip<Unsaved>;

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
    ) -> Result<NewPayslip, PayslipError> {
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
    use crate::payslip::{HourlyRate, WorkMinutes};
    use crate::project::{ProjectId, ProjectName};

    fn line(minutes: u32, rate: i64) -> PayslipLine {
        PayslipLine::new(
            ProjectId::from_i64(1).unwrap(),
            ProjectName::new("案件A").unwrap(),
            WorkMinutes::from_minutes(minutes).unwrap(),
            HourlyRate::from_yen(rate).unwrap(),
        )
        .unwrap()
    }

    fn staff() -> StaffId {
        StaffId::from_i64(1).unwrap()
    }

    fn period() -> PayPeriod {
        PayPeriod::new(2026, 9).unwrap()
    }

    #[allow(clippy::disallowed_methods, reason = "登録済みの作成中を作るため")]
    fn registered_draft(lines: Vec<PayslipLine>) -> DraftPayslip {
        let id = PayslipId::from_i64(7).unwrap();
        let Payslip::Draft(draft) =
            Payslip::reconstruct_draft(id, staff(), period(), lines).unwrap()
        else {
            unreachable!()
        };
        draft
    }

    #[test]
    fn empty_lines_are_rejected() {
        assert_eq!(
            Payslip::draft(staff(), period(), vec![]).unwrap_err(),
            PayslipError::EmptyLines
        );
    }

    #[test]
    fn too_many_lines_are_rejected() {
        let lines = vec![line(15, 1_000); 101];
        assert_eq!(
            Payslip::draft(staff(), period(), lines).unwrap_err(),
            PayslipError::TooManyLines { max: 100 }
        );
    }

    #[test]
    fn work_over_a_month_across_projects_is_rejected() {
        // 1件ずつは上限以内でも、合計が 744時間を超えればありえない
        let lines = vec![line(400 * 60, 1_000), line(400 * 60, 1_000)];
        assert_eq!(
            Payslip::draft(staff(), period(), lines).unwrap_err(),
            PayslipError::InvalidWorkMinutes
        );
    }

    #[test]
    fn total_sums_the_amounts_truncated_per_line() {
        // 1,002円 × 15分 ÷ 60 = 250.5円 → 250円 が2行。合算してから切り捨てる 501円 ではない
        let draft =
            Payslip::draft(staff(), period(), vec![line(15, 1_002), line(15, 1_002)]).unwrap();
        assert_eq!(draft.content().total().as_yen(), 500);
    }

    #[test]
    #[allow(clippy::disallowed_methods, reason = "記録から組み立て直すときの検証を確かめる")]
    fn reconstruct_rejects_empty_lines() {
        let id = PayslipId::from_i64(1).unwrap();
        assert_eq!(
            Payslip::reconstruct_draft(id, staff(), period(), vec![]).unwrap_err(),
            PayslipError::EmptyLines
        );
    }

    #[test]
    fn finalize_returns_the_finalized_payslip_and_the_event() {
        let draft = registered_draft(vec![line(600, 1_500)]);
        let at = datetime!(2026-09-30 10:00 UTC);

        let (finalized, event) = draft.finalize(at);

        assert_eq!(
            event,
            PayslipEvent::Finalized {
                payslip_id: PayslipId::from_i64(7).unwrap(),
                staff_id: staff(),
                period: period(),
                total: Money::from_yen(15_000).unwrap(),
                finalized_at: at,
            }
        );
        assert_eq!(finalized.finalized_at(), at);
        assert_eq!(Payslip::from(finalized).status(), PayslipStatus::Finalized);
    }
}
