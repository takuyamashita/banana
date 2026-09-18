//! 派遣社員(staff)。
//!
//! 派遣会社に登録し、派遣先の案件で働く人の雇用記録。給与明細はこの派遣社員ごとに作る。
//! 派遣社員は自分の給与明細を見るためにログインするので、ログイン用のアカウント(利用者)と
//! 1対1で結びつく。雇用記録を指す派遣社員番号と、アカウントを指す利用者IDは別のもの。

mod display_name;
mod entity;
mod error;
mod id;

pub use self::display_name::DisplayName;
pub use self::entity::{NewStaff, Staff};
pub use self::error::StaffError;
pub use self::id::StaffId;
