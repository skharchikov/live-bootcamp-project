use reqwest::cookie::Jar;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;

use auth_service::{
    app_state::{AppState, BannedTokenStoreType, TwoFactorAuthCodeStoreType},
    config::{AppConfig, CorsConfig, PostgresConfig, RedisConfig},
    get_postgres_pool, get_redis_client,
    services::{
        HashMapTwoFactorAuthCodeStore, MockEmailClient, PostgresUserStore, RedisBannedTokenStore,
    },
    Application, LoginRequest, SignupRequest, VerifyTokenRequest,
};
use fake::{faker::internet::en::SafeEmail, Fake};
use serde::Serialize;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use testcontainers_modules::{
    postgres::Postgres as PostgresImage,
    redis::Redis as RedisImage,
    testcontainers::{runners::AsyncRunner, ContainerAsync},
};

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFactorAuthCodeStoreType,
    pub email_client: Arc<RwLock<MockEmailClient>>,
    pub pg_config: PostgresConfig,
    pub cleanup_called: bool,
    // Container handles owned by the test so their `Drop` runs while the
    // tokio runtime is still alive — avoids leaking Docker containers on
    // process exit.
    _pg_container: ContainerAsync<PostgresImage>,
    _redis_container: ContainerAsync<RedisImage>,
}

// Throttle concurrent container starts so the Docker daemon does not
// thrash when the test harness fans out across many threads.
static CONTAINER_START_SEM: Semaphore = Semaphore::const_new(4);

impl TestApp {
    pub async fn new() -> Self {
        let _permit = CONTAINER_START_SEM
            .acquire()
            .await
            .expect("Container start semaphore closed");
        let pg_container = PostgresImage::default()
            .start()
            .await
            .expect("Failed to start Postgres container");
        let pg_host = pg_container
            .get_host()
            .await
            .expect("Failed to read Postgres host")
            .to_string();
        let pg_port = pg_container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to read Postgres port");

        let redis_container = RedisImage::default()
            .start()
            .await
            .expect("Failed to start Redis container");
        let redis_host = redis_container
            .get_host()
            .await
            .expect("Failed to read Redis host")
            .to_string();
        let redis_port = redis_container
            .get_host_port_ipv4(6379)
            .await
            .expect("Failed to read Redis port");

        let pg_config = PostgresConfig {
            host: pg_host,
            port: pg_port,
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            db: Uuid::new_v4().to_string(),
            max_connections: 5,
        };
        let redis_config = RedisConfig {
            host: redis_host,
            port: redis_port,
            password: String::new(),
        };
        let pg_pool = configure_postgresql(&pg_config).await;
        let redis_connection = Arc::new(RwLock::new(configure_redis(&redis_config)));

        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_token_store =
            Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_connection)));
        let two_fa_code_store = Arc::new(RwLock::new(HashMapTwoFactorAuthCodeStore::default()));
        let email_client = Arc::new(RwLock::new(MockEmailClient::default()));
        let app_state = AppState::new(
            user_store,
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client.clone(),
        );
        let config = AppConfig {
            host: "127.0.0.1".parse().unwrap(),
            port: 0, // Use port 0 to let the OS assign an available port.
            cors: CorsConfig {
                allowed_origins: "http://localhost:8000".to_string(),
            },
            postgres: pg_config.clone(),
            redis: redis_config,
        };
        let app = Application::build(app_state, config)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        Self {
            address,
            cookie_jar,
            http_client,
            banned_token_store,
            two_fa_code_store,
            email_client,
            pg_config,
            cleanup_called: false,
            _pg_container: pg_container,
            _redis_container: redis_container,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn login(&self, login_body: &LoginRequest) -> reqwest::Response {
        self.post_impl("/login", login_body).await
    }

    pub async fn signup(&self, signup_body: &SignupRequest) -> reqwest::Response {
        self.post_impl("/signup", signup_body).await
    }

    pub async fn post_signup<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        self.post_impl("/signup", body).await
    }

    pub async fn post_login<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        self.post_impl("/login", body).await
    }

    pub async fn post_verify_2fa<Body: Serialize>(&self, body: &Body) -> reqwest::Response {
        self.post_impl("/verify-2fa", body).await
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn verify_token(&self, body: &VerifyTokenRequest) -> reqwest::Response {
        self.post_impl("/verify-token", &body).await
    }

    pub async fn post_impl<Body>(&self, path: &str, body: &Body) -> reqwest::Response
    where
        Body: Serialize,
    {
        self.http_client
            .post(format!("{}{}", &self.address, path))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn cleanup(&mut self) {
        if !self.cleanup_called {
            delete_database(&self.pg_config, &self.pg_config.db).await;
            self.cleanup_called = true;
        }
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        // We can't make this async, so we just log a warning if cleanup wasn't called.
        if !self.cleanup_called {
            eprintln!("Warning: TestApp was dropped without calling cleanup. Database may not have been deleted.");
        }
    }
}

fn configure_redis(redis_config: &RedisConfig) -> redis::Connection {
    get_redis_client(redis_config)
        .expect("Failed to create Redis client")
        .get_connection()
        .expect("Failed to connect to Redis")
}

async fn configure_postgresql(pg_config: &PostgresConfig) -> PgPool {
    configure_database(pg_config).await;

    get_postgres_pool(pg_config)
        .await
        .expect("Failed to create Postgres connection pool!")
}

async fn configure_database(pg_config: &PostgresConfig) {
    // Create the per-test database via the admin (`postgres`) database.
    let admin_url = pg_config.connection_string_with_db("postgres");
    let admin_pool = PgPoolOptions::new()
        .connect(&admin_url)
        .await
        .expect("Failed to create Postgres admin connection pool.");

    admin_pool
        .execute(format!(r#"CREATE DATABASE "{}";"#, pg_config.db).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate the new database.
    let db_pool = PgPoolOptions::new()
        .connect(&pg_config.connection_string())
        .await
        .expect("Failed to create per-test connection pool.");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database");
}

async fn delete_database(pg_config: &PostgresConfig, db_name: &str) {
    // Connect to the default `postgres` admin database so we can drop the
    // target database without holding a connection to it.
    let postgresql_conn_url = pg_config.connection_string_with_db("postgres");

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}

pub fn get_random_email() -> String {
    SafeEmail().fake()
}
