use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::services::hashmap_user_service::HashmapUserStore;
use auth_service::{app_state::AppState, Application};

#[tokio::main]
async fn main() {
    let rw_user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    let app_state = AppState::new(rw_user_store);
    let app = Application::build(app_state, "0.0.0.0:3000")
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}
