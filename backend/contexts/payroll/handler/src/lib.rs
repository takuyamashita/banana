//! payroll の外からの入口。
//!
//! - gRPC: proto の型と domain の型の相互変換、ロール判定(認可の入口)、usecase のエラーを
//!   gRPC のステータスに翻訳する
//! - 出来事: 勤怠(timesheet)から届いた出来事を proto の型で読み、承認された稼働を記録する usecase を呼ぶ

mod error;
mod events;
mod payroll;
mod project;
mod staff;

pub use events::TimesheetEventHandler;
pub use payroll::PayrollServiceHandler;
pub use project::ProjectServiceHandler;
pub use staff::StaffServiceHandler;
