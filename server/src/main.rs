/// Route handlers for all API endpoints.
mod routes;

/// Setup utilities for logging and database connections.
mod setup;

use crate::routes::auth::login::login;
use crate::routes::auth::register::register;
use crate::setup::{init_logging, setup_db};
use axum::routing::post;
use axum::{Router, routing::get};
use sqlx::PgPool;
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

    let app = create_router(setup_db().await);

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

#[inline(always)]
fn create_router(pool: PgPool) -> Router {
    // Health check route
    let health_routes = Router::new().route("/api/health", get(|| async { "ok :)" }));

    // Authentication routes
    let auth_routes = Router::new()
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login));

    Router::new()
        .merge(health_routes)
        .merge(auth_routes)
        .with_state(pool)
}
