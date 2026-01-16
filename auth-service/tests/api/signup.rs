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
