//! 型で保証している業務ルール

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

/// 未登録の給与明細は番号を持たず、確定もできない(作成して登録してから確定する)。
///
/// ```compile_fail
/// use payroll_domain::payslip::{HourlyRate, PayPeriod, Payslip, PayslipId, PayslipLine, WorkMinutes};
/// use payroll_domain::project::ProjectId;
/// use payroll_domain::staff::StaffId;
/// use time::OffsetDateTime;
///
/// let line = PayslipLine::new(
///     ProjectId::from_i64(1).unwrap(),
///     WorkMinutes::from_minutes(600).unwrap(),
///     HourlyRate::from_yen(1_500).unwrap(),
/// )
/// .unwrap();
/// let staff = StaffId::from_i64(1).unwrap();
/// let period = PayPeriod::new(2026, 9).unwrap();
/// let draft = Payslip::draft(staff, period, vec![line]).unwrap();
/// draft.content().id();
/// ```
///
/// ```compile_fail
/// use payroll_domain::payslip::{HourlyRate, PayPeriod, Payslip, PayslipId, PayslipLine, WorkMinutes};
/// use payroll_domain::project::ProjectId;
/// use payroll_domain::staff::StaffId;
/// use time::OffsetDateTime;
///
/// let line = PayslipLine::new(
///     ProjectId::from_i64(1).unwrap(),
///     WorkMinutes::from_minutes(600).unwrap(),
///     HourlyRate::from_yen(1_500).unwrap(),
/// )
/// .unwrap();
/// let staff = StaffId::from_i64(1).unwrap();
/// let period = PayPeriod::new(2026, 9).unwrap();
/// let draft = Payslip::draft(staff, period, vec![line]).unwrap();
/// draft.finalize(OffsetDateTime::UNIX_EPOCH);
/// ```
///
/// ```
/// use payroll_domain::payslip::{HourlyRate, PayPeriod, Payslip, PayslipId, PayslipLine, WorkMinutes};
/// use payroll_domain::project::ProjectId;
/// use payroll_domain::staff::StaffId;
/// use time::OffsetDateTime;
///
/// let line = PayslipLine::new(
///     ProjectId::from_i64(1).unwrap(),
///     WorkMinutes::from_minutes(600).unwrap(),
///     HourlyRate::from_yen(1_500).unwrap(),
/// )
/// .unwrap();
/// let staff = StaffId::from_i64(1).unwrap();
/// let period = PayPeriod::new(2026, 9).unwrap();
/// let id = PayslipId::from_i64(1).unwrap();
/// let Payslip::Draft(draft) = Payslip::reconstruct_draft(id, staff, period, vec![line]).unwrap() else {
///     unreachable!()
/// };
/// assert_eq!(draft.content().id(), id);
/// let (_finalized, _event) = draft.finalize(OffsetDateTime::UNIX_EPOCH);
/// ```
struct UnsavedPayslipCannotBeFinalized;

/// 確定できるのは作成中の給与明細だけ。作成中か確定済みか分からないまま確定しようとすると通らず、
/// 確定済みの給与明細はもう一度確定できない。
///
/// ```compile_fail
/// use payroll_domain::payslip::{HourlyRate, PayPeriod, Payslip, PayslipId, PayslipLine, WorkMinutes};
/// use payroll_domain::project::ProjectId;
/// use payroll_domain::staff::StaffId;
/// use time::OffsetDateTime;
///
/// let line = PayslipLine::new(
///     ProjectId::from_i64(1).unwrap(),
///     WorkMinutes::from_minutes(600).unwrap(),
///     HourlyRate::from_yen(1_500).unwrap(),
/// )
/// .unwrap();
/// let staff = StaffId::from_i64(1).unwrap();
/// let period = PayPeriod::new(2026, 9).unwrap();
/// let id = PayslipId::from_i64(1).unwrap();
/// let payslip = Payslip::reconstruct_draft(id, staff, period, vec![line]).unwrap();
/// payslip.finalize(OffsetDateTime::UNIX_EPOCH);
/// ```
///
/// ```compile_fail
/// use payroll_domain::payslip::{HourlyRate, PayPeriod, Payslip, PayslipId, PayslipLine, WorkMinutes};
/// use payroll_domain::project::ProjectId;
/// use payroll_domain::staff::StaffId;
/// use time::OffsetDateTime;
///
/// let line = PayslipLine::new(
///     ProjectId::from_i64(1).unwrap(),
///     WorkMinutes::from_minutes(600).unwrap(),
///     HourlyRate::from_yen(1_500).unwrap(),
/// )
/// .unwrap();
/// let staff = StaffId::from_i64(1).unwrap();
/// let period = PayPeriod::new(2026, 9).unwrap();
/// let id = PayslipId::from_i64(1).unwrap();
/// let Payslip::Draft(draft) = Payslip::reconstruct_draft(id, staff, period, vec![line]).unwrap() else {
///     unreachable!()
/// };
/// let (finalized, _event) = draft.finalize(OffsetDateTime::UNIX_EPOCH);
/// finalized.finalize(OffsetDateTime::UNIX_EPOCH);
/// ```
///
/// ```
/// use payroll_domain::payslip::{HourlyRate, PayPeriod, Payslip, PayslipId, PayslipLine, WorkMinutes};
/// use payroll_domain::project::ProjectId;
/// use payroll_domain::staff::StaffId;
/// use time::OffsetDateTime;
///
/// let line = PayslipLine::new(
///     ProjectId::from_i64(1).unwrap(),
///     WorkMinutes::from_minutes(600).unwrap(),
///     HourlyRate::from_yen(1_500).unwrap(),
/// )
/// .unwrap();
/// let staff = StaffId::from_i64(1).unwrap();
/// let period = PayPeriod::new(2026, 9).unwrap();
/// let id = PayslipId::from_i64(1).unwrap();
/// let payslip = Payslip::reconstruct_draft(id, staff, period, vec![line]).unwrap();
/// if let Payslip::Draft(draft) = payslip {
///     let (_finalized, _event) = draft.finalize(OffsetDateTime::UNIX_EPOCH);
/// }
/// ```
struct OnlyDraftPayslipCanBeFinalized;

/// 番号は種類ごとに別の型で、給与明細番号を派遣社員番号として渡すことはできない。
///
/// ```compile_fail
/// use payroll_domain::payslip::{PayslipId, PayPeriod, Payslip};
///
/// let payslip_id = PayslipId::from_i64(1).unwrap();
/// let _ = Payslip::draft(payslip_id, PayPeriod::new(2026, 9).unwrap(), vec![]);
/// ```
///
/// ```
/// use payroll_domain::payslip::{PayPeriod, Payslip};
/// use payroll_domain::staff::StaffId;
///
/// let staff_id = StaffId::from_i64(1).unwrap();
/// let _ = Payslip::draft(staff_id, PayPeriod::new(2026, 9).unwrap(), vec![]);
/// ```
struct IdsOfDifferentKindsCannotBeMixed;
