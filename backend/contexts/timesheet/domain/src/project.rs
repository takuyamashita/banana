//! 案件(project)。
//!
//! 派遣社員が稼働する仕事の単位。給与(payroll)で登録された案件の写しで、勤怠では登録も変更もしない

use thiserror::Error;

platform_kernel::positive_id! {
    /// 案件番号。給与(payroll)で振られた番号をそのまま使う
    pub struct ProjectId;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProjectError {
    #[error("案件名は1〜100文字で入力してください")]
    InvalidName,
}

/// 案件名
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectName(String);

impl ProjectName {
    pub fn new(name: impl Into<String>) -> Result<Self, ProjectError> {
        let name = name.into();
        let length = name.trim().chars().count();
        if length == 0 || length > 100 {
            return Err(ProjectError::InvalidName);
        }
        Ok(Self(name))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 派遣社員が稼働する案件
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    id: ProjectId,
    name: ProjectName,
}

impl Project {
    #[must_use]
    pub fn new(id: ProjectId, name: ProjectName) -> Self {
        Self { id, name }
    }

    #[must_use]
    pub fn id(&self) -> ProjectId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &ProjectName {
        &self.name
    }
}
