//! payroll の gRPC ハンドラ。
//!
//! proto の型と domain の型の相互変換、ロール判定(認可の入口)、usecase のエラーを
//! gRPC のステータスに翻訳することがこの層の仕事。

mod error;
mod payroll;
mod project;
mod staff;

pub use payroll::PayrollServiceHandler;
pub use project::ProjectServiceHandler;
pub use staff::StaffServiceHandler;
