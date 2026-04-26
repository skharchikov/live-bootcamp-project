use std::error::Error;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
};

#[derive(Debug, Clone, PartialEq)]
pub struct HashedPassword(String);

impl HashedPassword {
    pub async fn parse(pwd: &str) -> Result<Self, String> {
        if Self::validate_password(pwd).is_ok() {
            let password_hash = Self::compute_password_hash(pwd)
                .await
                .map_err(|e| format!("Failed to compute password hash: {}", e))?;
            Ok(HashedPassword(password_hash))
        } else {
            Err("Password does not meet the required criteria".to_string())
        }
    }

    fn validate_password(pwd: &str) -> Result<(), String> {
        let contains_number = pwd.chars().any(|c| c.is_numeric());
        let contains_uppercase = pwd.chars().any(|c| c.is_uppercase());
        let contains_special = pwd.chars().any(|c| !c.is_alphanumeric());
        let contains_three_chars_in_a_row = pwd
            .as_bytes()
            .windows(3)
            .any(|w| w[0] == w[1] && w[1] == w[2]);

        if contains_three_chars_in_a_row {
            Err("Password must not contain three identical characters in a row".to_string())
        } else if !contains_special {
            Err("Password must contain at least one special character".to_string())
        } else if !contains_number {
            Err("Password must contain at least one numeric character".to_string())
        } else if !contains_uppercase {
            Err("Password must contain at least one uppercase letter".to_string())
        } else if pwd.len() < 8 {
            Err("Password must be at least 8 characters long".to_string())
        } else {
            Ok(())
        }
    }

    pub fn parse_password_hash(hash: &str) -> Result<Self, String> {
        let password_hash = PasswordHash::new(hash)
            .map_err(|_| format!("Provided string is not a valid hashed password: {}", hash))?;

        Ok(HashedPassword(password_hash.to_string()))
    }

    async fn compute_password_hash(password: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let password = password.to_owned();
        let result = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

            Ok(password_hash)
        })
        .await?;

        result
    }

    pub async fn verify_raw_password(
        &self,
        password_candidate: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let password_hash = self.as_ref().to_owned();
        let password_candidate = password_candidate.to_owned();
        let result = tokio::task::spawn_blocking(move || {
            let expected_password_hash: PasswordHash<'_> = PasswordHash::new(&password_hash)?;

            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                .map_err(|_| "Password verification failed".into())
        })
        .await;

        result?
    }
}

impl AsRef<str> for HashedPassword {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::Password as FakePassword;
    use fake::Fake;
    use quickcheck::Gen;
    use rand::SeedableRng;

    use super::*;

    #[tokio::test]
    async fn short_password_is_rejected() {
        let pwd = "Ab1!";
        assert!(HashedPassword::parse(pwd).await.is_err());
    }

    #[tokio::test]
    async fn password_without_number_is_rejected() {
        let pwd = "Abcdefg!";
        assert!(HashedPassword::parse(pwd).await.is_err());
    }

    #[tokio::test]
    async fn password_without_uppercase_is_rejected() {
        let pwd = "abcdef1!";
        assert!(HashedPassword::parse(pwd).await.is_err());
    }

    #[tokio::test]
    async fn password_without_special_character_is_rejected() {
        let pwd = "Abcdef12";
        assert!(HashedPassword::parse(pwd).await.is_err());
    }

    #[tokio::test]
    async fn password_with_three_identical_characters_in_a_row_is_rejected() {
        let pwd = "Abbbcd1!";
        assert!(HashedPassword::parse(pwd).await.is_err());
    }

    #[tokio::test]
    async fn valid_password_is_accepted() {
        let pwd = "Abcde1!f";
        assert!(HashedPassword::parse(pwd).await.is_ok());
    }

    #[tokio::test]
    async fn can_parse_valid_argon2_hash() {
        let raw_password = "Test1234!";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        );

        let hash_string = argon2
            .hash_password(raw_password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        let hash_password = HashedPassword::parse_password_hash(&hash_string).unwrap();

        assert_eq!(hash_password.as_ref(), hash_string);
        assert!(hash_password.as_ref().starts_with("$argon2id$v=19$"));
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub String);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let seed: u64 = g.size() as u64;
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let password: String = FakePassword(8..30).fake_with_rng(&mut rng);
            let required_characters = "A1!";
            let password = format!("{}{}", password, required_characters);
            Self(password)
        }
    }

    #[tokio::test]
    #[quickcheck_macros::quickcheck]
    async fn valid_passwords_are_parsed_successfully(valid_password: ValidPasswordFixture) -> bool {
        HashedPassword::parse(&valid_password.0).await.is_ok()
    }
}
