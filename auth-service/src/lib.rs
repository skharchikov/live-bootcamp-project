use std::error::Error;

use axum::{
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    serve::Serve,
    Json, Router,
};
use redis::{Client, RedisResult};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir};

pub mod app_state;
pub mod config;
pub mod domain;
pub mod routes;
pub mod services;
pub mod utils;

pub use domain::*;
pub use routes::*;
pub use utils::*;

use crate::{
    app_state::AppState,
    config::{AppConfig, PostgresConfig, RedisConfig},
};

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, config: AppConfig) -> Result<Self, Box<dyn Error>> {
        let allowed_origins = config
            .cors
            .allowed_origins()
            .into_iter()
            .map(|origin| origin.parse())
            .collect::<Result<Vec<_>, _>>()?;

        let cors = CorsLayer::new()
            // Allow GET and POST requests
            .allow_methods([Method::GET, Method::POST])
            // Allow cookies to be included in requests
            .allow_credentials(true)
            .allow_origin(allowed_origins);

        let router = Router::new()
            .fallback_service(ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/verify-2fa", post(verify_2fa))
            .route("/logout", post(logout))
            .route("/verify-token", post(verify_token))
            .with_state(app_state)
            .layer(cors); // Add CORS config to our Axum router

        let address = format!("{}:{}", config.host, config.port);
        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Self { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid auth token"),
            AuthAPIError::TokenBanned => (StatusCode::UNAUTHORIZED, "Invalid auth token"),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Missing auth token"),
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::IncorrectCredentials => {
                (StatusCode::UNAUTHORIZED, "Incorrect credentials")
            }
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
        };
        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });
        (status, body).into_response()
    }
}

pub async fn get_postgres_pool(config: &PostgresConfig) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections.into())
        .connect(&config.connection_string())
        .await
}

pub fn get_redis_client(redis_config: &RedisConfig) -> RedisResult<Client> {
    Client::open(redis_config.connection_string())
}
