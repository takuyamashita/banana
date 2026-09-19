//! 振込依頼(payout)。
//!
//! 確定した給与明細の支給額を、派遣社員の口座へ振り込むよう振込先に依頼した記録。
//! 1つの給与明細につき依頼は1回で、振込先が受け付けたか、断ったかを残す。
//! 断られた依頼は、管理者が理由を確かめて対処する。

mod entity;
mod id;
mod outcome;

pub use self::entity::{NewPayout, Payout};
pub use self::id::PayoutId;
pub use self::outcome::PayoutOutcome;
