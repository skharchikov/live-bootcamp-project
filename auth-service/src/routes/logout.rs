use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode};
use axum_extra::extract::{cookie::Cookie, CookieJar};

use crate::Token;
use crate::{app_state::AppState, validate_token, AuthAPIError, JWT_COOKIE_NAME};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = jar.get(JWT_COOKIE_NAME);
    let cookie = match cookie {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIError::MissingToken)),
    };

    let token = cookie.value();

    let _ = match validate_token(token).await {
        Ok(claims) => claims,
        Err(_) => return (jar, Err(AuthAPIError::InvalidToken)),
    };

    let mut store = state.banned_token_store.write().await;
    if store.add_token(Token(token.to_string())).await.is_err() {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }
    let jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));
    (jar, Ok(StatusCode::OK))
}
