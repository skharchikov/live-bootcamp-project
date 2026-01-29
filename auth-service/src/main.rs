use auth_service::{app_state::AppState, Application};

#[tokio::main]
async fn main() {
    let app_state = AppState::default();
    let app = Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
