use crate::{validate_token, AuthAPIError};

use axum::{http::StatusCode, Json};
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

pub async fn verify_token(
    Json(request): Json<VerifyTokenRequest>,
) -> Result<StatusCode, AuthAPIError> {
    match validate_token(&request.token).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(AuthAPIError::InvalidToken),
    }
}
