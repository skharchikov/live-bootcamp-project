use crate::{Email, LoginAttemptId, Password, Token, TwoFACode, User};

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Debug, PartialEq)]
pub enum BannedTokenStoreError {
    TokenAlreadyExists,
    UnexpectedError,
}

#[derive(Debug, PartialEq)]
pub enum TwoFactorAuthCodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
}

#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn add_token(&mut self, token: Token) -> Result<(), BannedTokenStoreError>;
    async fn contains(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}

#[async_trait::async_trait]
pub trait TwoFactorAuthCodeStore: Send + Sync {
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFactorAuthCodeStoreError>;
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        two_fa_code: TwoFACode,
    ) -> Result<(), TwoFactorAuthCodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFactorAuthCodeStoreError>;
}
