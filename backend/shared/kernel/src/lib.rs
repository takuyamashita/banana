//! どのコンテキストでも同じ意味で使う、最小限の共通の値。
//!
//! - 金額([`Money`])
//! - 番号([`positive_id!`] で宣言する型)と、未登録の目印([`Unsaved`])
//! - メールアドレス([`Email`])
//! - 利用者([`UserId`]・[`Role`]・[`AuthenticatedUser`])

mod email;
mod id;
mod money;
mod user;

pub use crate::email::{Email, InvalidEmail};
pub use crate::id::{InvalidId, Unsaved};
pub use crate::money::{Money, MoneyError};
pub use crate::user::{AuthenticatedUser, InvalidUserId, Role, UserId};
