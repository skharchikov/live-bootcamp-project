use reqwest::cookie::Jar;
use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::{
    app_state::AppState,
    config::{AppConfig, CorsConfig},
    services::hashmap_user_service::HashmapUserStore,
    Application, LoginRequest, SignupRequest,
};
use serde::Serialize;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
}

#[derive(Serialize)]
pub struct Verify2FABody {
    #[serde(rename = "2FACode")]
    pub code: String,
    pub login_attempt_id: String,
    pub email: String,
}

impl Verify2FABody {
    pub fn new(code: String, login_attempt_id: String, email: String) -> Self {
        Self {
            code,
            login_attempt_id,
            email,
        }
    }
}

#[derive(Serialize)]
pub struct VerifyTokenBody {
    pub token: String,
}

impl VerifyTokenBody {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

impl TestApp {
    pub async fn new() -> Self {
        let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
        let app_state = AppState::new(user_store);
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

    pub async fn verify_2fa(&self, verify_2fa_body: Verify2FABody) -> reqwest::Response {
        self.post_impl("/verify-2fa", &verify_2fa_body).await
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn verify_token(&self, body: VerifyTokenBody) -> reqwest::Response {
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
