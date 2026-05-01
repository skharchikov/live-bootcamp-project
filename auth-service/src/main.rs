use auth_service::config::{AppConfig, PostgresConfig, RedisConfig};
use auth_service::{get_postgres_pool, get_redis_client};
use redis::Connection;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

use auth_service::services::{
    MockEmailClient, PostgresUserStore, RedisBannedTokenStore, RedisTwoFactorAuthCodeStore,
};
use auth_service::{app_state::AppState, Application};

#[tokio::main]
async fn main() {
    init_tracing();

    let config = AppConfig::load().expect("Failed to load configuration");
    tracing::info!(
        host = %config.host,
        port = %config.port,
        cors_allowed_origins = %config.cors.allowed_origins,
        postgres_host = %config.postgres.host,
        postgres_user = %config.postgres.username,
        postgres_db = %config.postgres.db,
        "Configuration loaded"
    );
    let pg_pool = configure_postgresql(&config.postgres).await;
    let redis_connection = Arc::new(RwLock::new(configure_redis(&config.redis).await));

    let rw_user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let rw_banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(
        redis_connection.clone(),
    )));
    let rw_two_fa_code_store = Arc::new(RwLock::new(RedisTwoFactorAuthCodeStore::new(
        redis_connection.clone(),
    )));
    let email_client = Arc::new(RwLock::new(MockEmailClient::default()));
    let app_state = AppState::new(
        rw_user_store,
        rw_banned_token_store,
        rw_two_fa_code_store,
        email_client,
    );
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

async fn configure_postgresql(config: &PostgresConfig) -> PgPool {
    let pg_pool = get_postgres_pool(config)
        .await
        .expect("Failed to create PostgreSQL connection pool");

    sqlx::migrate!("./migrations")
        .run(&pg_pool)
        .await
        .expect("Failed to run database migrations");

    pg_pool
}

async fn configure_redis(config: &RedisConfig) -> Connection {
    get_redis_client(config)
        .expect("Failed to create Redis connection")
        .get_connection()
        .expect("Failed to connect to Redis")
}
