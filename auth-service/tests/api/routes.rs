use fake::{
    faker::internet::ar_sa::{Password, SafeEmail},
    Fake,
};

use crate::helpers::{LoginBody, SignupBody, TestApp, Verify2FABody, VerifyTokenBody};

// Tokio's test macro is used to run the test in an async environment
#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;

    let response = app.get_root().await;

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
}

#[tokio::test]
async fn signup_returns_201_when_valid_data_provided() {
    let app = TestApp::new().await;
    let email: String = SafeEmail().fake();
    let password: String = Password(8..12).fake();
    let signup_body = SignupBody::new(password, email, false);

    let response = app.signup(signup_body).await;

    assert_eq!(response.status().as_u16(), 201);
}

#[tokio::test]
async fn login_returns_200_when_valid_data_provided() {
    let app = TestApp::new().await;
    let email: String = SafeEmail().fake();
    let password: String = Password(8..12).fake();
    let login_body = LoginBody::new(email, password);

    let response = app.login(login_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn logout_returns_200_when_valid_jwt_provided() {
    let app = TestApp::new().await;

    let response = app.logout("valid_jwt_token").await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_2fa_returns_200_when_valid_data_provided() {
    let app = TestApp::new().await;
    let email: String = SafeEmail().fake();
    let code: String = Password(3..6).fake();
    let verify_2fa_body = Verify2FABody::new(code, "attempt_id".to_owned(), email);

    let response = app.verify_2fa(verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_token_returns_200_when_valid_data_provided() {
    let app = TestApp::new().await;
    let body = VerifyTokenBody::new("valid_jwt_token".to_owned());

    let response = app.verify_token(body).await;

    assert_eq!(response.status().as_u16(), 200);
}
