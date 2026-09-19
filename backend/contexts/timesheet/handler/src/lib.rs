//! timesheet の外からの入口。
//!
//! - gRPC: proto の型と domain の型の相互変換、ロール判定(認可の入口)、usecase のエラーを
//!   gRPC のステータスに翻訳する
//! - 出来事: 給与(payroll)から届いた出来事を proto の型で読み、写しを記録する usecase を呼ぶ

mod error;
mod events;
mod timesheet;

pub use events::PayrollEventHandler;
pub use timesheet::TimesheetServiceHandler;
