use platform_kernel::{Money, Unsaved};

use super::{PayoutId, PayoutOutcome};
use crate::payslip::PayslipId;
use crate::staff::StaffId;

/// 振込依頼。確定した給与明細の支給額を振り込むよう依頼し、振込先が答えた結果
#[derive(Debug)]
pub struct Payout<Id = PayoutId> {
    /// 振込依頼番号
    id: Id,
    /// 支給額が決まった給与明細
    payslip_id: PayslipId,
    /// 振込を受ける派遣社員
    staff_id: StaffId,
    /// 振り込む金額(給与明細の支給額)
    amount: Money,
    /// 振込先が受け付けたか、断ったか
    outcome: PayoutOutcome,
}

/// まだ記録していない振込依頼
pub type NewPayout = Payout<Unsaved>;

impl<Id> Payout<Id> {
    #[must_use]
    pub fn payslip_id(&self) -> PayslipId {
        self.payslip_id
    }

    #[must_use]
    pub fn staff_id(&self) -> StaffId {
        self.staff_id
    }

    #[must_use]
    pub fn amount(&self) -> Money {
        self.amount
    }

    #[must_use]
    pub fn outcome(&self) -> &PayoutOutcome {
        &self.outcome
    }
}

impl Payout<Unsaved> {
    /// 振込先が答えた振込依頼を作る
    #[must_use]
    pub fn new(
        payslip_id: PayslipId,
        staff_id: StaffId,
        amount: Money,
        outcome: PayoutOutcome,
    ) -> Self {
        Self { id: Unsaved, payslip_id, staff_id, amount, outcome }
    }
}

impl Payout<PayoutId> {
    /// 記録済みの振込依頼を、記録されている内容から組み立て直す
    #[must_use]
    pub fn reconstruct(
        id: PayoutId,
        payslip_id: PayslipId,
        staff_id: StaffId,
        amount: Money,
        outcome: PayoutOutcome,
    ) -> Self {
        Self { id, payslip_id, staff_id, amount, outcome }
    }

    #[must_use]
    pub fn id(&self) -> PayoutId {
        self.id
    }
}
