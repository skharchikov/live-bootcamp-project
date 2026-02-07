use auth_service::{ErrorResponse, LoginRequest, SignupRequest, JWT_COOKIE_NAME};
use fake::{faker::internet::en::SafeEmail, Fake};
use reqwest::Url;

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let response = app.logout().await;

    assert_eq!(
        response.status().as_u16(),
        400,
        "The API did not return a 400 BAD REQUEST",
    );

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME);

    assert!(auth_cookie.is_none());

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Missing auth token".to_owned()
    );
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.logout().await;

    assert_eq!(response.status().as_u16(), 401);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME);

    assert!(auth_cookie.is_none());

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid auth token".to_owned()
    );
}

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    let app = TestApp::new().await;

    let email: String = SafeEmail().fake();
    let signup_body = SignupRequest::new("ValidPass1!".to_string(), email.clone(), false);

    let response = app.signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = LoginRequest::new(email, "ValidPass1!".to_string());
    let response = app.login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let response = app.logout().await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;

    let email: String = SafeEmail().fake();
    let signup_body = SignupRequest::new("ValidPass1!".to_string(), email.clone(), false);

    let response = app.signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = LoginRequest::new(email, "ValidPass1!".to_string());
    let response = app.login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let response = app.logout().await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(auth_cookie.value().is_empty());

    let response = app.logout().await;
    assert_eq!(response.status().as_u16(), 400);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Missing auth token".to_owned()
    );
}
