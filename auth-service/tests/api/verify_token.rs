use crate::helpers::{TestApp, VerifyTokenBody};

#[tokio::test]
async fn verify_token_returns_200_when_valid_data_provided() {
    let app = TestApp::new().await;
    let body = VerifyTokenBody::new("valid_jwt_token".to_owned());

    let response = app.verify_token(body).await;

    assert_eq!(response.status().as_u16(), 200);
}
