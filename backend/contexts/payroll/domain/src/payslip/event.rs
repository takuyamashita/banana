use platform_kernel::Money;
use time::OffsetDateTime;

use super::{PayPeriod, PayslipId};
use crate::staff::StaffId;

/// 給与明細に起きた、他の業務が知るべき出来事。
///
/// 起きた出来事は必ず記録し、振込などの後続の業務に知らせる
#[must_use = "給与明細の出来事は記録して後続の業務に知らせる必要がある"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayslipEvent {
    /// 給与明細が確定し、支給額が決まった。振込はこれを受けて行う
    Finalized {
        /// 確定した給与明細
        payslip_id: PayslipId,
        /// 支給を受ける派遣社員
        staff_id: StaffId,
        /// 対象月
        period: PayPeriod,
        /// 確定した支給額(全明細行の合計)
        total: Money,
        /// 確定した日時
        finalized_at: OffsetDateTime,
    },
}
