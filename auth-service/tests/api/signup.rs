use fake::{
    faker::internet::en::{Password, SafeEmail},
    Fake,
};

use crate::helpers::SignupBody;
use crate::helpers::TestApp;

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
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let email: String = SafeEmail().fake();

    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "one": "two",
            "three": 4,
            "five": 6
        }),
        serde_json::json!({
            "token": "1234567"
        }),
        serde_json::json!({
            "email": email
        }),
        serde_json::json!({
            "password": "password123",
        }),
        serde_json::json!({
            "requires2FA": true
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_impl("/signup", &test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
}
