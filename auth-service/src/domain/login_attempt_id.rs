use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: &str) -> Result<Self, String> {
        uuid::Uuid::parse_str(id)
            .map(|_| LoginAttemptId(id.to_string()))
            .map_err(|e| format!("Invalid LoginAttemptId: {}", e))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(uuid::Uuid::now_v7().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
