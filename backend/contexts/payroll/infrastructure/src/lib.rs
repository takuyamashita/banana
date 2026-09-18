//! payroll の infrastructure。domain の repository trait と usecase の ports を実装する

pub mod db;
pub mod external;
pub mod messaging;
pub mod repository;

pub use db::{MIGRATOR, connect};
