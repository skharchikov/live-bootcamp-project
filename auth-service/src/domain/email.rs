use validator::*;

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(email: &str) -> Result<Self, String> {
        email.try_into()
    }
}

impl TryFrom<&str> for Email {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.validate_email() {
            Ok(Email(value.to_string()))
        } else {
            Err(format!("Invalid email format: {}", value))
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use fake::{faker::internet::en::SafeEmail, Fake};
    use quickcheck::Arbitrary;
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn empty_string_is_rejected() {
        let email = "";
        assert!(Email::parse(email).is_err());
    }

    #[test]
    fn missing_at_symbol_is_rejected() {
        let email = "userdomain.com";
        assert!(Email::parse(email).is_err());
    }

    #[test]
    fn missing_subject_is_rejected() {
        let email = "@domain.com";
        assert!(Email::parse(email).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(String);

    impl Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let seed = g.size() as u64;
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            let email: String = SafeEmail().fake_with_rng(&mut rng);
            ValidEmailFixture(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        Email::parse(&valid_email.0).is_ok()
    }
}
