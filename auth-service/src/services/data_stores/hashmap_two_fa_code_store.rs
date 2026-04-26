use std::collections::HashMap;

use crate::{
    Email, LoginAttemptId, TwoFACode, TwoFactorAuthCodeStore, TwoFactorAuthCodeStoreError,
};

#[derive(Default)]
pub struct HashMapTwoFactorAuthCodeStore {
    store: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFactorAuthCodeStore for HashMapTwoFactorAuthCodeStore {
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFactorAuthCodeStoreError> {
        self.store
            .get(email)
            .cloned()
            .ok_or(TwoFactorAuthCodeStoreError::LoginAttemptIdNotFound)
    }

    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        two_fa_code: TwoFACode,
    ) -> Result<(), TwoFactorAuthCodeStoreError> {
        self.store.insert(email, (login_attempt_id, two_fa_code));
        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFactorAuthCodeStoreError> {
        self.store
            .remove(email)
            .map(|_| ())
            .ok_or(TwoFactorAuthCodeStoreError::LoginAttemptIdNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get_code() {
        let mut store = HashMapTwoFactorAuthCodeStore {
            store: HashMap::new(),
        };
        let email = Email::parse("safe@gmail.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        let result = store
            .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
            .await;

        assert!(result.is_ok());

        let retrieved = store.get_code(&email).await.unwrap();
        let (retrieved_login_attempt_id, retrieved_two_fa_code) = retrieved;

        assert_eq!(retrieved_login_attempt_id, login_attempt_id);
        assert_eq!(retrieved_two_fa_code, two_fa_code);
    }

    #[tokio::test]
    async fn test_remove_code() {
        let mut store = HashMapTwoFactorAuthCodeStore {
            store: HashMap::new(),
        };
        let email = Email::parse("safe@gmail.com").unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        let _ = store
            .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
            .await;

        let remove_result = store.remove_code(&email).await;
        let get_result = store.get_code(&email).await;

        assert!(remove_result.is_ok());
        assert!(get_result.is_err());
    }
}
