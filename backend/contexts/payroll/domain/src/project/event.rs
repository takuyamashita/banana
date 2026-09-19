use super::{ProjectId, ProjectName};

/// 案件に起きた、他の業務が知るべき出来事
#[must_use = "案件の出来事は記録して他の業務に知らせる必要がある"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectEvent {
    /// 案件が登録された。他の業務(勤怠など)は、これで派遣社員が働く案件を知る
    Created {
        /// 振られた案件番号
        project_id: ProjectId,
        /// 案件名
        name: ProjectName,
    },
}
