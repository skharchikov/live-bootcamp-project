#[derive(Debug, Clone, PartialEq)]
pub struct Password(String);

impl Password {
    pub fn parse(pwd: &str) -> Result<Self, String> {
        let contains_number = pwd.chars().any(|c| c.is_numeric());
        let contains_uppercase = pwd.chars().any(|c| c.is_uppercase());
        let contains_special = pwd.chars().any(|c| !c.is_alphanumeric());
        let contains_three_chars_in_a_row = pwd
            .as_bytes()
            .windows(3)
            .any(|w| w[0] == w[1] && w[1] == w[2]);

        if contains_three_chars_in_a_row {
            return Err(
                "Password must not contain three identical characters in a row".to_string(),
            );
        }

        if !contains_special {
            return Err("Password must contain at least one special character".to_string());
        }

        if !contains_number {
            return Err("Password must contain at least one numeric character".to_string());
        }

        if !contains_uppercase {
            return Err("Password must contain at least one uppercase letter".to_string());
        }

        if pwd.len() < 8 {
            Err("Password must be at least 8 characters long".to_string())
        } else {
            Ok(Password(pwd.to_string()))
        }
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

mod tests {
    pub use super::*;

    #[test]
    fn short_password_is_rejected() {
        let pwd = "Ab1!";
        assert!(Password::parse(pwd).is_err());
    }

    #[test]
    fn password_without_number_is_rejected() {
        let pwd = "Abcdefg!";
        assert!(Password::parse(pwd).is_err());
    }

    #[test]
    fn password_without_uppercase_is_rejected() {
        let pwd = "abcdef1!";
        assert!(Password::parse(pwd).is_err());
    }

    #[test]
    fn password_without_special_character_is_rejected() {
        let pwd = "Abcdef12";
        assert!(Password::parse(pwd).is_err());
    }

    #[test]
    fn password_with_three_identical_characters_in_a_row_is_rejected() {
        let pwd = "Abbbcd1!";
        assert!(Password::parse(pwd).is_err());
    }

    #[test]
    fn valid_password_is_accepted() {
        let pwd = "Abcde1!f";
        assert!(Password::parse(pwd).is_ok());
    }
}
