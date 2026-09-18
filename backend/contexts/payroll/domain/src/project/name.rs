use super::ProjectError;

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
