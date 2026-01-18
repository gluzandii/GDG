/// Route handlers for all API endpoints.
mod routes;

/// Setup utilities for logging and database connections.
mod setup;

use crate::routes::auth::login::api_auth_login_post;
use crate::routes::auth::register::api_auth_register_post;
use crate::routes::chats::codes::delete::api_chats_codes_delete;
use crate::routes::chats::codes::post::api_chats_codes_post;
use crate::routes::chats::messages::delete::api_chats_messages_delete;
use crate::routes::chats::messages::get::api_chats_messages_get;
use crate::routes::chats::messages::patch::api_chats_messages_patch;
use crate::routes::chats::post::api_chats_post;
use crate::routes::chats::ws::api_chats_ws;
use crate::routes::users::get::api_users_get;
use crate::routes::users::patch::api_users_patch;
use crate::setup::{init_logging, setup_db};
use ::middleware::auth_middleware;
use axum::middleware;
use axum::routing::{any, post};
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
    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0".into());
    let addr = format!("{}:{}", bind_addr, port);

    if let Err(e) = env::var("DATABASE_URL") {
        println!(
            "An error occurred while trying to retrive DATABASE_URL env variable: {}",
            e
        );
        std::process::exit(1);
    }

    if let Err(e) = env::var("JWT_SECRET_KEY") {
        println!(
            "An error occurred while trying to retrive JWT_SECRET_KEY env variable: {}",
            e
        );
        std::process::exit(1);
    }

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
        .route("/api/auth/register", post(api_auth_register_post))
        .route("/api/auth/login", post(api_auth_login_post));

    // Protected user routes (auth required)
    let protected_users_routes = Router::new()
        .route("/api/users", get(api_users_get).patch(api_users_patch))
        .layer(middleware::from_fn(auth_middleware));

    // Protected chat routes (auth required)
    let protected_chat_routes = Router::new()
        .route(
            "/api/chats",
            post(api_chats_post), // Submit the chat code
        )
        .route(
            "/api/chats/codes",
            post(api_chats_codes_post).delete(api_chats_codes_delete),
        )
        .route(
            "/api/chats/messages",
            get(api_chats_messages_get)
                .delete(api_chats_messages_delete)
                .patch(api_chats_messages_patch),
        )
        .route("/api/chats/ws", any(api_chats_ws))
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
