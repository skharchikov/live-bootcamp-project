use reqwest::cookie::Jar;
use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::{
    app_state::{AppState, BannedTokenStoreType, TwoFactorAuthCodeStoreType},
    config::{AppConfig, CorsConfig},
    services::{
        HashMapTwoFactorAuthCodeStore, HashmapUserStore, HashsetBannedTokenStore, MockEmailClient,
    },
    Application, LoginRequest, SignupRequest, Verify2FARequest, VerifyTokenRequest,
};
use fake::{faker::internet::en::SafeEmail, Fake};
use serde::Serialize;

pub fn get_random_email() -> String {
    SafeEmail().fake()
}

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFactorAuthCodeStoreType,
    pub email_client: Arc<RwLock<MockEmailClient>>,
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
        let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
        let two_fa_code_store = Arc::new(RwLock::new(HashMapTwoFactorAuthCodeStore::default()));
        let email_client = Arc::new(RwLock::new(MockEmailClient::default()));
        let app_state = AppState::new(
            user_store,
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client.clone(),
        );
        let config = AppConfig {
            host: "127.0.0.1".parse().unwrap(),
            port: 0, // Use port 0 to let the OS assign an available port.
            cors: CorsConfig {
                allowed_origins: "http://localhost:8000".to_string(),
            },
        };
        let app = Application::build(app_state, config)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        Self {
            address,
            cookie_jar,
            http_client,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn login(&self, login_body: &LoginRequest) -> reqwest::Response {
        self.post_impl("/login", login_body).await
    }

    pub async fn signup(&self, signup_body: &SignupRequest) -> reqwest::Response {
        self.post_impl("/signup", signup_body).await
    }

    pub async fn verify_2fa(&self, verify_2fa_body: &Verify2FARequest) -> reqwest::Response {
        self.post_impl("/verify-2fa", verify_2fa_body).await
    }

    pub async fn post_signup<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        self.post_impl("/signup", body).await
    }

    pub async fn post_login<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        self.post_impl("/login", body).await
    }

    pub async fn post_verify_2fa<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        self.post_impl("/verify-2fa", body).await
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn verify_token(&self, body: &VerifyTokenRequest) -> reqwest::Response {
        self.post_impl("/verify-token", &body).await
    }

    pub async fn post_impl<Body>(&self, path: &str, body: &Body) -> reqwest::Response
    where
        Body: Serialize,
    {
        self.http_client
            .post(format!("{}{}", &self.address, path))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}
