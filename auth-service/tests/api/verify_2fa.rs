use fake::{
    faker::internet::en::{Password, SafeEmail},
    Fake,
};

use crate::helpers::{TestApp, Verify2FABody};

#[tokio::test]
async fn verify_2fa_returns_200_when_valid_data_provided() {
    let app = TestApp::new().await;
    let email: String = SafeEmail().fake();
    let code: String = Password(3..6).fake();
    let verify_2fa_body = Verify2FABody::new(code, "attempt_id".to_owned(), email);

    let response = app.verify_2fa(verify_2fa_body).await;

    assert_eq!(response.status().as_u16(), 200);
}
