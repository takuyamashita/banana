//! 案件(project)。
//!
//! 派遣社員が派遣先で従事する仕事の単位。給与明細の各行は、どの案件での稼働かを案件で示す。

mod entity;
mod error;
mod event;
mod id;
mod name;

pub use self::entity::{NewProject, Project};
pub use self::error::ProjectError;
pub use self::event::ProjectEvent;
pub use self::id::ProjectId;
pub use self::name::ProjectName;
