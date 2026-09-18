//! 給与計算(payroll)の業務。
//!
//! 派遣会社が、派遣社員ごとに毎月の給与を計算して確定するまでを扱う。
//! - 案件(project): 派遣社員が従事する仕事の単位
//! - 派遣社員(staff): 派遣先で働く人の雇用記録と、ログイン用アカウントの対応
//! - 給与明細(payslip): 派遣社員1人の1か月分の給与。案件ごとの稼働と時給から支給額を決める
//!
//! それぞれは番号で互いを指し、ほかのものの中身を直接は持たない

pub mod payslip;
pub mod project;
pub mod staff;

/// まだ登録していないことを表す目印。
///
/// 案件・派遣社員・給与明細の番号は、登録したときに初めて決まる。そのため登録前のものは番号を持たず、
/// 番号の位置にこの目印を入れる(`Project<Unsaved>` など)。番号を尋ねられるのは登録済みのものだけ。
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
