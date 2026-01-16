/// Route handlers for all API endpoints.
mod routes;

/// Setup utilities for logging and database connections.
mod setup;

use crate::routes::auth::login::login_route;
use crate::routes::auth::register::register_route;
use crate::routes::chats::delete_code::delete_code_chat_route;
use crate::routes::chats::get_chats_route;
use crate::routes::chats::new_code::new_chat_route;
use crate::routes::chats::submit_code::submit_code_chat_route;
use crate::routes::chats::ws::ws_handler;
use crate::routes::users::me::me_route;
use crate::routes::users::update::update_route;
use crate::routes::users::update_password::update_password_route;
use crate::setup::{init_logging, setup_db};
use ::middleware::auth_middleware;
use axum::middleware;
use axum::routing::{any, delete, patch, post};
use axum::{Router, routing::get};
use sqlx::PgPool;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::{KeyExtractor, SmartIpKeyExtractor};

#[tokio::main]
async fn main() {
    init_logging();
    #[cfg(debug_assertions)]
    if let Err(_) = dotenvy::dotenv() {
        tracing::warn!("Failed to load .env file. Continuing without it.");
    }

    let port = env::var("PORT").unwrap_or_else(|_| "2607".into());
    let addr = format!("127.0.0.1:{}", port);

    let app = create_router(setup_db().await).into_make_service_with_connect_info::<SocketAddr>();

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
    let mut rate_limit_config = GovernorConfigBuilder::default();
    rate_limit_config.per_second(1).burst_size(20);

    let rate_limit_layer = GovernorLayer::new(Arc::new(
        rate_limit_config
            .key_extractor(IpRouteKeyExtractor)
            .finish()
            .expect("Failed to build rate limiter config"),
    ));

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
        .route("/api/users/update-password", patch(update_password_route))
        .layer(middleware::from_fn(auth_middleware));

    // Protected chat routes (auth required)
    let protected_chat_routes = Router::new()
        .route("/api/chats/new-code", post(new_chat_route))
        .route("/api/chats/delete-code", delete(delete_code_chat_route))
        .route("/api/chats", get(get_chats_route))
        .route("/api/chats/submit-code", post(submit_code_chat_route))
        .route("/api/chats/ws", any(ws_handler))
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .merge(health_routes)
        .merge(auth_routes)
        .merge(protected_users_routes)
        .merge(protected_chat_routes)
        .with_state(pool)
        .layer(rate_limit_layer)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct IpRouteKey {
    ip: IpAddr,
    path: String,
}

#[derive(Clone, Copy, Debug)]
struct IpRouteKeyExtractor;

impl KeyExtractor for IpRouteKeyExtractor {
    type Key = IpRouteKey;

    fn name(&self) -> &'static str {
        "ip+route"
    }

    fn extract<T>(
        &self,
        req: &axum::http::Request<T>,
    ) -> Result<Self::Key, tower_governor::GovernorError> {
        let ip = SmartIpKeyExtractor.extract(req)?;
        Ok(IpRouteKey {
            ip,
            path: req.uri().path().to_owned(),
        })
    }

    fn key_name(&self, key: &Self::Key) -> Option<String> {
        Some(format!("{} {}", key.ip, key.path))
    }
}
