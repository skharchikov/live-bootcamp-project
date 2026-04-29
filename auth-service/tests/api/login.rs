use crate::helpers::TestApp;
use auth_service::{
    Email, ErrorResponse, LoginRequest, SignupRequest, TwoFactorAuthResponse, JWT_COOKIE_NAME,
};
use fake::{faker::internet::en::SafeEmail, Fake};
use serde_json::json;
use test_helpers::api_test;

#[api_test]
async fn should_return_422_if_malformed_credentials() {
    let input = json!({
        "email": "not-an-email"
    });
    let response = app.post_impl("/login", &input).await;
    assert_eq!(response.status().as_u16(), 422);
}

#[tokio::test]
async fn should_return_400_if_incorrect_credentials() {
    let app = TestApp::new().await;
    let input = json!({
        "email": "not-an-email",
        "password": "12345678"
    });
    let response = app.post_impl("/login", &input).await;
    assert_eq!(response.status().as_u16(), 400);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid credentials".to_owned()
    );
}

#[tokio::test]
async fn should_return_401_if_invalid_input() {
    let app = TestApp::new().await;
    let email: String = SafeEmail().fake();

    let input = serde_json::json!({
        "email": email,
        "password": "ValidPass1!",
        "requires2FA": true
    });

    let response = app.post_impl("/signup", &input).await;
    assert_eq!(response.status().as_u16(), 201);

    let input = json!({
        "email": email,
        "password": "InvalidPass1!"
    });

    let response = app.post_impl("/login", &input).await;
    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Incorrect credentials".to_owned()
    )
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
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
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let app = TestApp::new().await;

    let email: String = SafeEmail().fake();
    let signup_body = SignupRequest::new("ValidPass1!".to_string(), email.clone(), true);

    let response = app.signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = LoginRequest::new(email, "ValidPass1!".to_string());
    let response = app.login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    assert_eq!(
        response
            .json::<TwoFactorAuthResponse>()
            .await
            .expect("Could not deserialize response body to TwoFactorAuthResponse")
            .message,
        "2FA required".to_owned()
    );

    app.two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&login_body.email).unwrap())
        .await
        .expect("2FA code should be stored for the user");

    let email = Email::parse(&login_body.email).unwrap();
    app.email_client
        .read()
        .await
        .sent
        .read()
        .await
        .get(&email)
        .expect("Sent email should be recorded in the mock email client");
}
