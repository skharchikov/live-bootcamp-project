use std::net::IpAddr;

use confique::Config;

#[derive(Config, Debug)]
pub struct AppConfig {
    #[config(env = "APP_HOST")]
    pub host: IpAddr,
    #[config(env = "APP_PORT")]
    pub port: u16,
    #[config(nested)]
    pub cors: CorsConfig,
    #[config(nested)]
    pub postgres: PostgresConfig,
}

impl AppConfig {
    pub fn load() -> Result<AppConfig, confique::Error> {
        // Load environment variables from .env file.
        // Fails if .env file not found, not readable or invalid.
        match dotenvy::dotenv() {
            Ok(_) => (),
            Err(e) => {
                tracing::warn!("Failed to load .env file: {e}");
                // Continue without .env file, environment variables may still be set.
            }
        }
        AppConfig::builder().env().load()
    }
}

#[derive(Config, Debug)]
pub struct CorsConfig {
    #[config(env = "CORS_ALLOWED_ORIGINS")]
    pub allowed_origins: String,
}

impl CorsConfig {
    pub fn allowed_origins(&self) -> Vec<&str> {
        self.allowed_origins.split(',').map(str::trim).collect()
    }
}

#[derive(Config, Debug, Clone)]
pub struct PostgresConfig {
    #[config(env = "POSTGRES_HOST", default = "localhost")]
    pub host: String,
    #[config(env = "POSTGRES_USER")]
    pub username: String,
    #[config(env = "POSTGRES_PASSWORD")]
    pub password: String,
    #[config(env = "POSTGRES_DB")]
    pub db: String,
    #[config(env = "POSTGRES_MAX_CONNECTIONS", default = 5)]
    pub max_connections: u16,
}

impl PostgresConfig {
    pub fn connection_string(&self) -> String {
        self.connection_string_with_db(&self.db)
    }

    pub fn connection_string_with_db(&self, db: &str) -> String {
        format!(
            "postgres://{}:{}@{}:5432/{}",
            self.username, self.password, self.host, db
        )
    }
}
