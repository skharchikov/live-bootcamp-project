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
