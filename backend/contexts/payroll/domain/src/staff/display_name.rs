use super::StaffError;

/// 表示名。画面で派遣社員を見分けるための名前(氏名など)で、前後の空白を除いて1〜50文字
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayName(String);

impl DisplayName {
    pub fn new(value: impl Into<String>) -> Result<Self, StaffError> {
        let value = value.into().trim().to_owned();
        if value.is_empty() || value.chars().count() > 50 {
            return Err(StaffError::InvalidDisplayName);
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
    fn trimmed_and_1_to_50_characters() {
        assert_eq!(DisplayName::new(" 派遣 太郎 ").unwrap().as_str(), "派遣 太郎");
        assert!(DisplayName::new("あ".repeat(50)).is_ok());
        assert_eq!(DisplayName::new("あ".repeat(51)), Err(StaffError::InvalidDisplayName));
        assert_eq!(DisplayName::new("   "), Err(StaffError::InvalidDisplayName));
    }
}
