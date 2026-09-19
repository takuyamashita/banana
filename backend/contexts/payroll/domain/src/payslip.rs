//! 給与明細(payslip)。
//!
//! 派遣社員1人の、ある1か月分の給与を表す。案件ごとの稼働時間と時給を明細行として持ち、
//! その合計が支給額になる。管理者が内容を確かめて「確定」すると、以後は変更できず、
//! 確定した事実を振込などの後続業務に知らせる。

mod entity;
mod error;
mod event;
mod id;
mod line;
mod pay_period;
mod work_minutes;

pub use self::entity::{
    Draft, DraftPayslip, Finalized, FinalizedPayslip, NewPayslip, Payslip, PayslipIn, PayslipStatus,
};
pub use self::error::PayslipError;
pub use self::event::PayslipEvent;
pub use self::id::PayslipId;
pub use self::line::PayslipLine;
pub use self::pay_period::PayPeriod;
pub use self::work_minutes::WorkMinutes;
