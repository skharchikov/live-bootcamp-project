use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, AuthAPIError, Email, HashedPassword, User, UserStoreError};

#[derive(Deserialize, Serialize, Debug)]
pub struct SignupRequest {
    pub password: String,
    pub email: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

impl SignupRequest {
    pub fn new(password: String, email: String, requires_2fa: bool) -> Self {
        Self {
            password,
            email,
            requires_2fa,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}

#[tracing::instrument(name = "Signup", skip(state, request))]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(&request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let hashed_password = HashedPassword::parse(&request.password)
        .await
        .map_err(|_| AuthAPIError::InvalidCredentials)?;
    let user = User::new(email, hashed_password, request.requires_2fa);

    let mut user_store = state.user_store.write().await;
    user_store.add_user(user).await.map_err(|e| match e {
        UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
        _ => AuthAPIError::UnexpectedError,
    })?;

    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}
