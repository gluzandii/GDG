/// Route handlers for all API endpoints.
mod routes;

/// Setup utilities for logging and database connections.
mod setup;

use crate::routes::auth::login::login_route;
use crate::routes::auth::register::register_route;
use crate::routes::users::me::me_route;
use crate::routes::users::update::update_route;
use crate::routes::users::update_password::update_password_route;
use crate::setup::{init_logging, setup_db};
use ::middleware::auth_middleware;
use axum::middleware;
use axum::routing::{patch, post};
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
    // Health check route (no auth required)
    let health_routes = Router::new().route("/api/health", get(|| async { "ok :)" }));

    // Authentication routes (no auth required)
    let auth_routes = Router::new()
        .route("/api/auth/register", post(register_route))
        .route("/api/auth/login", post(login_route));

    // Protected user routes (auth required)
    let protected_users_routes = Router::new()
        .route("/api/users/me", get(me_route))
        .route("/api/users/update", patch(update_route))
        .route("/api/users/update-password", post(update_password_route))
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .merge(health_routes)
        .merge(auth_routes)
        .merge(protected_users_routes)
        .with_state(pool)
}
