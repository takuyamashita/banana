//! payroll コンテキストのドメイン。集約ごとにモジュールを分け、集約間は ID で参照する

pub mod payslip;
pub mod project;
pub mod repository;
pub mod staff;
