use auth_service::config::AppConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::services::{HashmapUserStore, HashsetBannedTokenStore};
use auth_service::{app_state::AppState, Application};

#[tokio::main]
async fn main() {
    init_tracing();

    let config = AppConfig::load().expect("Failed to load configuration");
    tracing::info!("Configuration loaded: {:?}", &config);

    let rw_user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    let rw_banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let app_state = AppState::new(rw_user_store, rw_banned_token_store);
    let app = Application::build(app_state, config)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

fn init_tracing() {
    use tracing_subscriber::prelude::*;
    let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);
    tracing_subscriber::registry().with(fmt_layer).init();
}
