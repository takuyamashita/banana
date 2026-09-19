//! 型で保証していることを確かめる doc テスト(`cargo test --doc` で実行される)

/// 未登録の案件は番号を持たない。
///
/// ```compile_fail
/// use payroll_domain::project::{NewProject, ProjectName};
///
/// let project = NewProject::new(ProjectName::new("案件A").unwrap());
/// project.id();
/// ```
///
/// ```
/// use payroll_domain::project::{Project, ProjectId, ProjectName};
///
/// let id = ProjectId::from_i64(1).unwrap();
/// let project = Project::reconstruct(id, ProjectName::new("案件A").unwrap());
/// assert_eq!(project.id(), id);
/// ```
struct UnsavedHasNoId;

/// 確定済みの給与明細は、もう一度確定できない。
///
/// ```compile_fail
/// use payroll_domain::payslip::{PayPeriod, Payslip, PayslipLine, WorkMinutes};
/// use payroll_domain::project::ProjectId;
/// use payroll_domain::staff::StaffId;
/// use platform_kernel::Money;
/// use time::OffsetDateTime;
///
/// let line = PayslipLine::new(
///     ProjectId::from_i64(1).unwrap(),
///     WorkMinutes::from_minutes(600).unwrap(),
///     Money::from_yen(1_500).unwrap(),
/// );
/// let period = PayPeriod::new(2026, 9).unwrap();
/// let draft = Payslip::draft(StaffId::from_i64(1).unwrap(), period, vec![line]).unwrap();
/// let (finalized, _event) = draft.finalize(OffsetDateTime::UNIX_EPOCH);
/// finalized.finalize(OffsetDateTime::UNIX_EPOCH);
/// ```
///
/// ```
/// use payroll_domain::payslip::{PayPeriod, Payslip, PayslipLine, WorkMinutes};
/// use payroll_domain::project::ProjectId;
/// use payroll_domain::staff::StaffId;
/// use platform_kernel::Money;
/// use time::OffsetDateTime;
///
/// let line = PayslipLine::new(
///     ProjectId::from_i64(1).unwrap(),
///     WorkMinutes::from_minutes(600).unwrap(),
///     Money::from_yen(1_500).unwrap(),
/// );
/// let period = PayPeriod::new(2026, 9).unwrap();
/// let draft = Payslip::draft(StaffId::from_i64(1).unwrap(), period, vec![line]).unwrap();
/// let (_finalized, _event) = draft.finalize(OffsetDateTime::UNIX_EPOCH);
/// ```
struct FinalizedPayslipCannotBeFinalizedAgain;
