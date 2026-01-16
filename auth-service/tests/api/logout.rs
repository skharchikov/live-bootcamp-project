use crate::helpers::TestApp;

#[tokio::test]
async fn logout_returns_200_when_valid_jwt_provided() {
    let app = TestApp::new().await;

    let response = app.logout("valid_jwt_token").await;

    assert_eq!(response.status().as_u16(), 200);
}
