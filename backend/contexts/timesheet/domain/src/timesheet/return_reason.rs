use super::TimesheetError;

/// 差し戻しの理由。派遣社員が勤務表の何を直せばよいかを伝える
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnReason(String);

impl ReturnReason {
    pub fn new(reason: impl Into<String>) -> Result<Self, TimesheetError> {
        let reason = reason.into();
        let length = reason.trim().chars().count();
        if length == 0 || length > 500 {
            return Err(TimesheetError::InvalidReturnReason);
        }
        Ok(Self(reason))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
