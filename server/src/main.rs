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
    dotenvy::dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "2607".into());
    let addr = format!("127.0.0.1:{}", port);

    let pool = setup_db().await;

    let app = Router::new()
        .route("/health", get(health))
        .route("/auth/register", post(register))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok :)"
}
