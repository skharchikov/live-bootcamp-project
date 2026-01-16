use auth_service::Application;
use serde::Serialize;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

#[derive(Serialize)]
pub struct LoginBody {
    pub email: String,
    pub password: String,
}

impl LoginBody {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}

#[derive(Serialize)]
pub struct SignupBody {
    pub password: String,
    pub email: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

impl SignupBody {
    pub fn new(password: String, email: String, requires_2fa: bool) -> Self {
        Self {
            password,
            email,
            requires_2fa,
        }
    }
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
        let app = Application::build("127.0.0.1:0")
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::new();

        Self {
            address,
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

    pub async fn login(&self, login_body: LoginBody) -> reqwest::Response {
        self.http_client
            .post(format!("{}/login", &self.address))
            .json(&login_body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn signup(&self, signup_body: SignupBody) -> reqwest::Response {
        self.http_client
            .post(format!("{}/signup", &self.address))
            .json(&signup_body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn verify_2fa(&self, verify_2fa_body: Verify2FABody) -> reqwest::Response {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(&verify_2fa_body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn logout(&self, jwt_token: &str) -> reqwest::Response {
        self.http_client
            .post(format!("{}/logout", &self.address))
            .form(&[("jwt", jwt_token)])
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn verify_token(&self, body: VerifyTokenBody) -> reqwest::Response {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}
