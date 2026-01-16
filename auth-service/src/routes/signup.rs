use axum::{http::StatusCode, response::IntoResponse};

pub async fn signup() -> impl IntoResponse {
    StatusCode::CREATED.into_response()
}
