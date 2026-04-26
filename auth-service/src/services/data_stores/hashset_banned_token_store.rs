use std::collections::HashSet;

use crate::{BannedTokenStore, Token};

#[derive(Debug, Default)]
pub struct HashsetBannedTokenStore {
    pub tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&mut self, token: Token) -> Result<(), crate::BannedTokenStoreError> {
        self.tokens.insert(token.0);
        Ok(())
    }

    async fn contains(&self, token: &str) -> Result<bool, crate::BannedTokenStoreError> {
        Ok(self.tokens.contains(token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_contains_token() {
        let mut store = HashsetBannedTokenStore::default();
        let token = Token("test_token".to_string());

        store.add_token(token.clone()).await.unwrap();

        let contains = store.contains(&token.0).await.unwrap();
        assert!(contains);
    }

    #[tokio::test]
    async fn test_contains_nonexistent_token() {
        let store = HashsetBannedTokenStore::default();

        // Check if a non-existent token is in the store
        let contains = store.contains("nonexistent_token").await.unwrap();
        assert!(!contains);
    }
}
