//! Create new chat endpoint handler.
//!
//! Handles creation of new chat conversations.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sqlx::PgPool;
use utils::errors::error_response;

/// Handles chat creation requests.
///
/// This endpoint:
/// 1. Validates the request
/// 2. Creates a new chat in the database
/// 3. Returns the newly created chat details
///
/// # Arguments
///
/// * `pool` - The PostgreSQL connection pool
/// * `req` - The chat creation request
///
/// # Returns
///
/// - `201 CREATED` with chat details on success
/// - `400 BAD REQUEST` if validation fails
/// - `500 INTERNAL SERVER ERROR` if any server-side operation fails
#[tracing::instrument(skip(pool))]
pub async fn new_chat_route(State(pool): State<PgPool>) -> impl IntoResponse {
    // TODO: Implement chat creation logic

    tracing::info!("Creating new chat");

    // Placeholder response
    error_response(
        StatusCode::NOT_IMPLEMENTED,
        "Chat creation not yet implemented".to_string(),
    )
}
