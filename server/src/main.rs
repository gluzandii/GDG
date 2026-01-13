use axum::{routing::get, Router};
use std::env;
use std::path::Path;

#[cfg(debug_assertions)]
fn load_env() {
    match dotenvy::dotenv() {
        Ok(path) => println!("Loaded .env via dotenv(): {}", path.display()),
        Err(_) => {
            // fallback: load `.env` from the crate (server) directory
            let manifest = env!("CARGO_MANIFEST_DIR"); // points to `server` at compile time
            let env_path = Path::new(manifest).join(".env");
            match dotenvy::from_path(&env_path) {
                Ok(_) => println!("Loaded .env from {}", env_path.display()),
                Err(e) => println!("Failed to load {}: {}", env_path.display(), e),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    load_env();

    let port = env::var("PORT").unwrap_or_else(|_| "2607".into());
    let addr = format!("127.0.0.1:{}", port);

    let app = Router::new().route("/health", get(health));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "ok :)"
}
