use std::sync::Arc;

use payroll_domain::project::{NewProject, ProjectId, ProjectName};

use crate::UseCaseError;
use crate::ports::queries::{ProjectQuery, ProjectView};
use crate::ports::transaction::Transactions;

/// 管理者が案件を登録する
pub struct CreateProjectUseCase {
    transactions: Arc<dyn Transactions>,
}

impl CreateProjectUseCase {
    #[must_use]
    pub fn new(transactions: Arc<dyn Transactions>) -> Self {
        Self { transactions }
    }

    /// 案件を登録し、振られた案件番号を返す
    pub async fn execute(&self, name: ProjectName) -> Result<ProjectId, UseCaseError> {
        let mut tx = self.transactions.begin().await?;
        let id = tx.projects().insert(&NewProject::new(name)).await?;
        tx.commit().await?;
        Ok(id)
    }
}

/// 管理者が、登録済みの案件を一覧する
pub struct ListProjectsUseCase {
    query: Arc<dyn ProjectQuery>,
}

impl ListProjectsUseCase {
    #[must_use]
    pub fn new(query: Arc<dyn ProjectQuery>) -> Self {
        Self { query }
    }

    pub async fn execute(&self) -> Result<Vec<ProjectView>, UseCaseError> {
        Ok(self.query.list().await?)
    }
}
