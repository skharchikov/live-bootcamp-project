use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{BannedTokenStore, TwoFactorAuthCodeStore, UserStore};

pub type UserStoreType = Arc<RwLock<dyn UserStore>>;
pub type BannedTokenStoreType = Arc<RwLock<dyn BannedTokenStore>>;
pub type TwoFactorAuthCodeStoreType = Arc<RwLock<dyn TwoFactorAuthCodeStore>>;
pub type EmailClientType = Arc<dyn crate::EmailClient>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFactorAuthCodeStoreType,
    pub email_client: EmailClientType,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType,
        banned_token_store: BannedTokenStoreType,
        two_fa_code_store: TwoFactorAuthCodeStoreType,
        email_client: EmailClientType,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
    }
}
