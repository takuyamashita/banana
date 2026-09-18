//! domain の repository trait の MySQL 実装

mod payslip_repository;
mod project_repository;
mod staff_repository;

pub use payslip_repository::MySqlPayslipRepository;
pub use project_repository::MySqlProjectRepository;
pub use staff_repository::MySqlStaffRepository;
