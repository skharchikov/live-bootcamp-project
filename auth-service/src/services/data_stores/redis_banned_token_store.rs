use std::sync::Arc;

use redis::{Commands, Connection};
use tokio::sync::RwLock;

use crate::{utils::auth::TOKEN_TTL_SECONDS, BannedTokenStore, BannedTokenStoreError, Token};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn add_token(&mut self, token: Token) -> Result<(), BannedTokenStoreError> {
        let key = get_key(&token.0);
        let ttl: u64 = TOKEN_TTL_SECONDS
            .try_into()
            .map_err(|_| BannedTokenStoreError::UnexpectedError)?;

        let mut conn = self.conn.write().await;
        conn.set_ex::<_, _, ()>(key, true, ttl)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)
    }

    async fn contains(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = get_key(token);
        let mut conn = self.conn.write().await;
        conn.exists(key)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)
    }
}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
