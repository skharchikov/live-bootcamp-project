use std::sync::Arc;

use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
    Email, LoginAttemptId, TwoFACode, TwoFactorAuthCodeStore, TwoFactorAuthCodeStoreError,
};

pub struct RedisTwoFactorAuthCodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFactorAuthCodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFactorAuthCodeStore for RedisTwoFactorAuthCodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        two_fa_code: TwoFACode,
    ) -> Result<(), TwoFactorAuthCodeStoreError> {
        let key = get_key(&email);
        let tuple = TwoFATuple(
            login_attempt_id.as_ref().to_string(),
            two_fa_code.as_ref().to_string(),
        );
        let value = serde_json::to_string(&tuple)
            .map_err(|_| TwoFactorAuthCodeStoreError::UnexpectedError)?;

        let mut conn = self.conn.write().await;
        conn.set_ex::<_, _, ()>(key, value, TEN_MINUTES_IN_SECONDS)
            .map_err(|_| TwoFactorAuthCodeStoreError::UnexpectedError)
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFactorAuthCodeStoreError> {
        let key = get_key(email);
        let mut conn = self.conn.write().await;
        conn.del::<_, ()>(key)
            .map_err(|_| TwoFactorAuthCodeStoreError::UnexpectedError)
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFactorAuthCodeStoreError> {
        let key = get_key(email);
        let mut conn = self.conn.write().await;
        let value: String = conn
            .get(key)
            .map_err(|_| TwoFactorAuthCodeStoreError::LoginAttemptIdNotFound)?;

        let TwoFATuple(login_attempt_id, code) = serde_json::from_str(&value)
            .map_err(|_| TwoFactorAuthCodeStoreError::UnexpectedError)?;

        let login_attempt_id = LoginAttemptId::parse(&login_attempt_id)
            .map_err(|_| TwoFactorAuthCodeStoreError::UnexpectedError)?;
        let code = TwoFACode::parse(&code)
            .map_err(|_| TwoFactorAuthCodeStoreError::UnexpectedError)?;

        Ok((login_attempt_id, code))
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
