//! 案件(project)。
//!
//! 派遣社員が派遣先で従事する仕事の単位。給与明細の各行は、どの案件での稼働かを案件で示す。

use platform_kernel::Unsaved;
use thiserror::Error;

/// 案件の業務ルールに反したときの理由
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProjectError {
    /// 案件番号が正の数でない
    #[error("案件IDが不正です")]
    InvalidId,
    /// 案件名が空、または100文字を超えている
    #[error("案件名は1〜100文字で指定してください")]
    InvalidName,
}

/// 案件番号。登録された案件を一意に指す正の整数
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectId(i64);

impl ProjectId {
    pub fn from_i64(value: i64) -> Result<Self, ProjectError> {
        if value <= 0 {
            return Err(ProjectError::InvalidId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

/// 案件名。画面や給与明細で案件を見分けるための名前で、前後の空白を除いて1〜100文字
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectName(String);

impl ProjectName {
    pub fn new(value: impl Into<String>) -> Result<Self, ProjectError> {
        let value = value.into().trim().to_owned();
        if value.is_empty() || value.chars().count() > 100 {
            return Err(ProjectError::InvalidName);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 案件。派遣社員が従事する仕事の単位。
///
/// `Id` はまだ登録していない案件なら [`Unsaved`]、登録済みなら [`ProjectId`]。
/// 案件番号を尋ねられるのは登録済みの案件だけ:
///
/// ```compile_fail
/// use payroll_domain::project::{NewProject, ProjectName};
///
/// let project = NewProject::new(ProjectName::new("案件A").unwrap());
/// project.id(); // まだ登録していないので案件番号はない
/// ```
///
/// ```
/// use payroll_domain::project::{Project, ProjectId, ProjectName};
///
/// let id = ProjectId::from_i64(1).unwrap();
/// let project = Project::reconstruct(id, ProjectName::new("案件A").unwrap());
/// assert_eq!(project.id(), id);
/// ```
#[derive(Debug)]
pub struct Project<Id = ProjectId> {
    /// 案件番号
    id: Id,
    /// 案件名
    name: ProjectName,
}

/// まだ登録していない案件
pub type NewProject = Project<Unsaved>;

impl<Id> Project<Id> {
    #[must_use]
    pub fn name(&self) -> &ProjectName {
        &self.name
    }
}

impl Project<Unsaved> {
    /// 案件を新しく作る
    #[must_use]
    pub fn new(name: ProjectName) -> Self {
        Self { id: Unsaved, name }
    }
}

impl Project<ProjectId> {
    /// 登録済みの案件を、記録されている内容から組み立て直す
    #[must_use]
    pub fn reconstruct(id: ProjectId, name: ProjectName) -> Self {
        Self { id, name }
    }

    #[must_use]
    pub fn id(&self) -> ProjectId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_name_is_rejected() {
        assert_eq!(ProjectName::new("  "), Err(ProjectError::InvalidName));
    }

    #[test]
    fn name_is_trimmed() {
        assert_eq!(ProjectName::new(" 案件A ").unwrap().as_str(), "案件A");
    }
}
