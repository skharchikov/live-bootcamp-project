use crate::helpers::TestApp;
use auth_service::{
    ErrorResponse, LoginRequest, SignupRequest, VerifyTokenRequest, JWT_COOKIE_NAME,
};
use fake::{faker::internet::en::SafeEmail, Fake};

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let test_cases = vec![
        serde_json::json!({
            "token": true,
        }),
        serde_json::json!({}),
    ];

    for test_case in test_cases {
        let response = app.post_impl("/verify-token", &test_case).await;
        assert_eq!(response.status().as_u16(), 422);
    }
}

#[tokio::test]
async fn should_return_200_valid_token() {
    let app = TestApp::new().await;

    let email = SafeEmail().fake();
    let signup_body = SignupRequest::new("ValidPass1!".to_owned(), email, false);

    let response = app.signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = LoginRequest::new(signup_body.email.clone(), signup_body.password.clone());
    let response = app.login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let token = auth_cookie.value();

    let verify_token_body = VerifyTokenRequest::new(token.to_owned());
    let response = app.verify_token(&verify_token_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    let test_cases = vec!["", "invalid_token"];

    for test_case in test_cases {
        let verify_token_body = serde_json::json!({
            "token": test_case,
        });

        let response = app.post_impl("/verify-token", &verify_token_body).await;

        assert_eq!(response.status().as_u16(), 401);
        assert_eq!(
            response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse")
                .error,
            "Invalid auth token".to_owned()
        );
    }
}
