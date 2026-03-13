use axum::{extract::State, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::{generate_auth_cookie, AuthAPIError, Email, LoginAttemptId, TwoFACode};

#[derive(Serialize, Deserialize, Debug)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}

impl Verify2FARequest {
    pub fn new(email: String, login_attempt_id: String, two_fa_code: String) -> Self {
        Self {
            email,
            login_attempt_id,
            two_fa_code,
        }
    }
}

#[tracing::instrument(name = "Verify 2FA", skip(state, jar, request))]
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = match Email::parse(&request.email) {
        Ok(email) => email,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let login_attempt_id = match LoginAttemptId::parse(&request.login_attempt_id) {
        Ok(id) => id,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let two_fa_code = match TwoFACode::parse(&request.two_fa_code) {
        Ok(code) => code,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let mut code_store = state.two_fa_code_store.write().await;

    let stored = match code_store.get_code(&email).await {
        Ok(tuple) => tuple,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    if stored.0 != login_attempt_id || stored.1 != two_fa_code {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    if code_store.remove_code(&email).await.is_err() {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    let auth_cookie = match generate_auth_cookie(&email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    };

    let updated_jar = jar.add(auth_cookie);
    (updated_jar, Ok(()))
}
