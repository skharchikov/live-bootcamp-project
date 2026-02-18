use crate::{app_state::AppState, validate_token, AuthAPIError};

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

impl VerifyTokenRequest {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

#[tracing::instrument(name = "Verify Token", skip(state, request))]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<StatusCode, AuthAPIError> {
    state
        .banned_token_store
        .read()
        .await
        .contains(&request.token)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)
        .and_then(|is_banned| {
            if is_banned {
                Err(AuthAPIError::TokenBanned)
            } else {
                Ok(())
            }
        })?;

    match validate_token(&request.token).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AuthAPIError::InvalidToken),
    }
}
