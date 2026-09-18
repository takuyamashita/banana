//! payroll コンテキストのドメイン。集約ごとにモジュールを分け、集約間は ID で参照する

pub mod payslip;
pub mod project;
pub mod staff;

/// 未保存の集約の ID の位置に入れる目印。
///
/// ID は DB が採番するので、保存前の集約は ID を持たない。集約を `Project<Id = ProjectId>` のように
/// ID の型で引数化し、未保存は `Project<Unsaved>` で表す。`id()` は保存済みにだけ生えるので、
/// 未保存の集約から ID を読もうとするとコンパイルエラーになる。
///
/// ```compile_fail
/// use payroll_domain::project::{NewProject, ProjectName};
///
/// let project = NewProject::new(ProjectName::new("案件A").unwrap());
/// project.id(); // 未保存なので id() はない
/// ```
///
/// ```
/// use payroll_domain::project::{Project, ProjectId, ProjectName};
///
/// let project = Project::reconstruct(ProjectId::from_i64(1).unwrap(), ProjectName::new("案件A").unwrap());
/// assert_eq!(project.id().as_i64(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsaved;
