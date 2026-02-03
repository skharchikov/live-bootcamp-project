use crate::helpers::TestApp;
use auth_service::ErrorResponse;
use fake::{faker::internet::en::SafeEmail, Fake};
use serde_json::json;

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;
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
