mod routes;

use crate::routes::auth::register::register;
use axum::routing::post;
use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;

#[inline(always)]
async fn setup_db() -> Pool<Postgres> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&db_url)
        .await
        .expect("Failed to connect to Postgres");

    pool
}

#[tokio::main]
async fn main() {
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
