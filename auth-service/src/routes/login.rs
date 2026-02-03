use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::{AuthAPIError, Email, Password, UserStoreError};

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

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let user_store = state.user_store.read().await;
    let email = Email::parse(&request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(&request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    user_store
        .validate_user(&email, &password)
        .await
        .map_err(|e| match e {
            UserStoreError::UserNotFound | UserStoreError::InvalidCredentials => {
                AuthAPIError::IncorrectCredentials
            }
            _ => AuthAPIError::UnexpectedError,
        })?;

    user_store.get_user(&email).await.map_err(|e| match e {
        UserStoreError::UserNotFound => AuthAPIError::IncorrectCredentials,
        _ => AuthAPIError::UnexpectedError,
    })?;
    Ok(StatusCode::OK.into_response())
}
