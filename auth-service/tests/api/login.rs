use crate::helpers::{LoginBody, TestApp};
use fake::{
    faker::internet::en::{Password, SafeEmail},
    Fake,
};

#[tokio::test]
async fn login_returns_200_when_valid_data_provided() {
    let app = TestApp::new().await;
    let email: String = SafeEmail().fake();
    let password: String = Password(8..12).fake();
    let login_body = LoginBody::new(email, password);

    let response = app.login(login_body).await;

    assert_eq!(response.status().as_u16(), 200);
}
