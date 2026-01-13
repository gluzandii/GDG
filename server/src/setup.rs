//! Setup utilities for the server.
//!
//! This module contains initialization functions for database connections
//! and logging configuration.

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;

/// Sets up the PostgreSQL database connection pool.
///
/// Reads the `DATABASE_URL` environment variable and creates a connection pool
/// with the following configuration:
/// - Maximum connections: 5
/// - Acquire timeout: 5 seconds
///
/// # Returns
///
/// A PostgreSQL connection pool.
///
/// # Panics
///
/// Panics if:
/// - The `DATABASE_URL` environment variable is not set
/// - The connection to the database fails (logs error and exits with code 1)
pub(crate) async fn setup_db() -> Pool<Postgres> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&db_url)
        .await;

    let pool = match pool {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to connect to the database. Exiting.");
            std::process::exit(1);
        }
    };

    pool
}

use tracing_subscriber::{filter::Targets, fmt, prelude::*};

/// Initializes the tracing subscriber for application logging.
///
/// Configures logging with the following settings:
/// - Debug level for the `server` target in debug builds, INFO in release builds
/// - WARN level for `sqlx` and `tokio_util::compat` to reduce noise
/// - Pretty formatting with timestamps, levels, targets, file locations, thread names, IDs, and line numbers
/// - UTC time in RFC 3339 format
pub(crate) fn init_logging() {
    let server_level = if cfg!(debug_assertions) {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let filter = Targets::new()
        .with_target("server", server_level)
        .with_target("sqlx", tracing::Level::WARN)
        .with_target("tokio_util::compat", tracing::Level::WARN)
        .with_default(server_level);

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .pretty()
                .with_timer(fmt::time::UtcTime::rfc_3339())
                .with_level(true)
                .with_target(true)
                .with_file(true)
                .with_thread_names(true)
                .with_thread_ids(true)
                .with_line_number(true),
        )
        .init();
}
