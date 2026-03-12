use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: &str) -> Result<Self, String> {
        let code = code
            .parse::<u32>()
            .map_err(|e| format!("Invalid TwoFACode: {}", e))?;

        match code {
            100000..=999999 => Ok(TwoFACode(code.to_string())),
            _ => Err("TwoFACode must be a 6-digit number".to_string()),
        }
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let code = rand::rng().random_range(100000u32..=999999);
        TwoFACode(code.to_string())
    }
}
