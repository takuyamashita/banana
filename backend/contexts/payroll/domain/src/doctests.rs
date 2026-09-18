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
