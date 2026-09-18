use thiserror::Error;

/// 利用者IDが空、または長すぎる
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("利用者IDが不正です")]
pub struct InvalidUserId;

/// 利用者ID。ログインに使うアカウントを指す識別子。
///
/// アカウントは認証基盤が発行し、形式はそちらが決める(UUID など)。ここでは空でなく、
/// 64文字以内であることだけを求める
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn parse(value: impl Into<String>) -> Result<Self, InvalidUserId> {
        let value = value.into();
        if value.is_empty() || value.len() > 64 {
            return Err(InvalidUserId);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// アプリが扱うロール。プロバイダ固有のクレーム(`cognito:groups`・`realm_access.roles`)は
/// shared/auth の mapper でこれに詰め替える
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    Staff,
}

impl Role {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "admin" => Some(Self::Admin),
            "staff" => Some(Self::Staff),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Staff => "staff",
        }
    }
}

/// ログインしている利用者
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedUser {
    /// ログインに使っているアカウント
    pub user_id: UserId,
    /// 利用者に与えられているロール
    pub roles: Vec<Role>,
}

impl AuthenticatedUser {
    #[must_use]
    pub fn has_role(&self, role: Role) -> bool {
        self.roles.contains(&role)
    }
}
