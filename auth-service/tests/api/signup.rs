use auth_service::{ErrorResponse, SignupRequest, SignupResponse};
use fake::{faker::internet::en::SafeEmail, Fake};

use crate::helpers::TestApp;

#[tokio::test]
async fn should_return_201_when_valid_data_provided() {
    let app = TestApp::new().await;
    let email: String = SafeEmail().fake();
    let password = "ValidPass1!".to_owned();
    let signup_body = SignupRequest::new(password, email, false);

    let response = app.signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let expected_response = SignupResponse {
        message: "User created successfully!".to_owned(),
    };

    // Assert that we are getting the correct response body!
    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to SignupResponse"),
        expected_response
    );
}

#[tokio::test]
async fn shuld_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let email: String = SafeEmail().fake();

    let test_cases = [
        SignupRequest::new("".to_owned(), email.clone(), true),
        SignupRequest::new("password123".to_owned(), "".to_owned(), false),
        SignupRequest::new("password123".to_owned(), "invalidemail".to_owned(), true),
    ];

    for test_case in test_cases.iter() {
        let response = app.signup(test_case).await;
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

#[tokio::test]
async fn should_return_409_if_email_already_exist() {
    let app = TestApp::new().await;
    let email: String = SafeEmail().fake();

    let input = serde_json::json!({
        "email": email,
        "password": "ValidPass1!",
        "requires2FA": true
    });

    let response1 = app.post_impl("/signup", &input).await;
    let response2 = app.post_impl("/signup", &input).await;
    assert_eq!(response1.status().as_u16(), 201);
    assert_eq!(response2.status().as_u16(), 409);
    assert_eq!(
        response2
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
}
