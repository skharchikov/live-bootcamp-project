use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::{
    generate_auth_cookie, AuthAPIError, Email, EmailPayload, LoginAttemptId, TwoFACode,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl LoginRequest {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

#[tracing::instrument(name = "Login", skip(state, jar, request))]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let user_store = state.user_store.read().await;
    let email = match Email::parse(&request.email) {
        Ok(email) => email,
        Err(_) => {
            return (jar, Err(AuthAPIError::InvalidCredentials));
        }
    };

    if user_store
        .validate_user(&email, &request.password)
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let user = match user_store.get_user(&email).await {
        Ok(user) => user,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar),
    }
}

async fn handle_2fa(
    email: &Email,
    app_state: &AppState,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    if app_state
        .two_fa_code_store
        .write()
        .await
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    let content = format!("Your 2FA code is: {}", two_fa_code.as_ref());
    if app_state
        .email_client
        .write()
        .await
        .send_email(email, EmailPayload::TWO_FA_SUBJECT, &content)
        .await
        .is_err()
    {
        tracing::error!(
            "Error sending 2FA code email to {}: {}",
            email.as_ref(),
            two_fa_code.as_ref()
        );
        return (jar, Err(AuthAPIError::UnexpectedError));
    };
    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_string(),
        login_attempt_id: {
            let this = &login_attempt_id;
            this.as_ref().to_owned()
        },
    }));
    (jar, Ok((StatusCode::from_u16(206).unwrap(), response)))
}

fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let auth_cookie = match generate_auth_cookie(email) {
        Ok(cookie) => cookie,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    };

    let updated_jar = jar.add(auth_cookie);
    let response = LoginResponse::RegularAuth;

    (updated_jar, Ok((StatusCode::OK, Json(response))))
}
