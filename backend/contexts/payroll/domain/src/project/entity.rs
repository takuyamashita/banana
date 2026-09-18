use platform_kernel::Unsaved;

use super::{ProjectId, ProjectName};

/// 案件。派遣社員が従事する仕事の単位
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
