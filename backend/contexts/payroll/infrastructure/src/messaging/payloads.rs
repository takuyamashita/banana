// キューに流す JSON の形。domain の PayslipEvent には serde の derive を付けず、ここで詰め替える。
// キューの形が変わっても domain が引っ張られないようにするため
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayslipFinalizedPayload {
    pub staff_id: i64,
    pub pay_year: u16,
    pub pay_month: u8,
    pub total_yen: i64,
}

impl PayslipFinalizedPayload {
    pub const EVENT_TYPE: &'static str = "payslip.finalized";
}
