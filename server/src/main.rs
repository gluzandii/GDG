//! # GDG Realtime Chat Server
//!
//! This is the main server application for the GDG realtime chat platform.
//! It provides RESTful API endpoints for user authentication and real-time messaging.

/// Route handlers for all API endpoints.
mod routes;

/// Setup utilities for logging and database connections.
mod setup;

use crate::routes::auth::register::register;
use crate::setup::{init_logging, setup_db};
use axum::routing::post;
use axum::{Router, routing::get};
use std::env;

/// Main entry point for the server application.
///
/// Initializes logging, loads environment variables (in debug mode),
/// sets up the database connection pool, configures routes, and starts
/// the HTTP server.
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

/// Health check endpoint.
///
/// Returns a simple status message to verify the server is running.
///
/// # Returns
///
/// A static string "ok :)" indicating the server is healthy.
async fn health() -> &'static str {
    "ok :)"
}
