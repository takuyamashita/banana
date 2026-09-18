//! 案件(project)集約

use async_trait::async_trait;
use thiserror::Error;

use crate::Unsaved;
use crate::repository::RepositoryError;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProjectError {
    #[error("案件IDが不正です")]
    InvalidId,
    #[error("案件名は1〜100文字で指定してください")]
    InvalidName,
}

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

/// 案件。`Id` は保存済みなら `ProjectId`、未保存なら `Unsaved`
#[derive(Debug)]
pub struct Project<Id = ProjectId> {
    id: Id,
    name: ProjectName,
}

pub type NewProject = Project<Unsaved>;

impl<Id> Project<Id> {
    #[must_use]
    pub fn name(&self) -> &ProjectName {
        &self.name
    }
}

impl Project<Unsaved> {
    #[must_use]
    pub fn new(name: ProjectName) -> Self {
        Self { id: Unsaved, name }
    }
}

impl Project<ProjectId> {
    // 永続化からの再構築専用
    #[must_use]
    pub fn reconstruct(id: ProjectId, name: ProjectName) -> Self {
        Self { id, name }
    }

    #[must_use]
    pub fn id(&self) -> ProjectId {
        self.id
    }
}

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn insert(&self, new: &NewProject) -> Result<ProjectId, RepositoryError>;
    async fn list(&self) -> Result<Vec<Project>, RepositoryError>;
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
