use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::{cookie::Cookie, CookieJar};

use crate::{validate_token, AuthAPIError, JWT_COOKIE_NAME};

pub async fn logout(jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
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

    let jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));

    (jar, Ok(StatusCode::OK))
}
