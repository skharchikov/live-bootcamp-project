use std::error::Error;

use axum::{routing::post, serve::Serve, Router};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub mod app_state;
mod domain;
mod routes;
mod services;

pub use domain::*;
pub use routes::*;

use crate::app_state::AppState;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let assets_dir = ServeDir::new("assets");
        let router = Router::new()
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/verify-token", post(verify_token))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .with_state(app_state)
            .fallback_service(assets_dir);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Self { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}
