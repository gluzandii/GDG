mod routes;
mod setup;

use crate::routes::auth::register::register;
use crate::setup::{init_logging, setup_db};
use axum::routing::post;
use axum::{routing::get, Router};
use std::env;

#[tokio::main]
async fn main() {
    init_logging();
    #[cfg(debug_assertions)]
    if let Err(_) = dotenvy::dotenv() {
        tracing::warn!("Failed to load .env file. Continuing without it.");
    }

    let port = env::var("PORT").unwrap_or_else(|_| "2607".into());
    let addr = format!("127.0.0.1:{}", port);

    let pool = setup_db().await;

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(register))
        .with_state(pool);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to bind to address {}. Exiting.", addr);
            std::process::exit(1);
        }
    };

    println!("Listening on http://{}", addr);
    match axum::serve(listener, app).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!(error = ?e, "Error while running the server. Exiting.");
            std::process::exit(1);
        }
    }
}

async fn health() -> &'static str {
    "ok :)"
}
