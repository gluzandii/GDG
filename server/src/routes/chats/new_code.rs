//! Create new chat endpoint handler.
//!
//! Handles creation of new chat conversations.

use api_types::chats::new_code::CreateChatResponse;
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use utils::errors::error_response;

/// Handles chat creation requests.
///
/// This endpoint:
/// 1. Extracts the user ID from the authentication cookie
/// 2. Generates a unique random numeric code for the chat
/// 3. Creates a new chat code in the database linked to the user
/// 4. Returns the chat code
///
/// # Arguments
///
/// * `user_id` - The authenticated user's ID from the JWT cookie
/// * `pool` - The PostgreSQL connection pool
///
/// # Returns
///
/// - `201 CREATED` with the chat code on success
/// - `500 INTERNAL SERVER ERROR` if database operation fails
#[tracing::instrument(skip(pool, user_id))]
pub async fn new_chat_route(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    tracing::debug!(user_id, "Creating new chat code");

    // Generate a random 6-digit numeric code
    let code = generate_chat_code();

    // Insert the chat code into the database
    let result = sqlx::query!(
        r#"
        WITH user_chat_count AS (
            SELECT COUNT(*)::INT AS count FROM chat_codes WHERE user_id = $2
        )
        INSERT INTO chat_codes (code, user_id)
        SELECT $1, $2
        FROM user_chat_count
        WHERE user_chat_count.count < 5
        "#,
        code as i32,
        user_id
    )
    .execute(&pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() == 1 => {}
        Ok(_) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "You already have 5 chat codes.".to_string(),
            );
        }
        Err(e) => {
            tracing::error!(error = ?e, user_id, "Failed to create chat code");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create chat code".to_string(),
            );
        }
    }

    tracing::info!(user_id, code, "Chat code created successfully");
    (StatusCode::CREATED, Json(CreateChatResponse { code })).into_response()
}

/// Generates a random 5-digit numeric code for chat identification.
#[inline(always)]
fn generate_chat_code() -> u16 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(10000..u16::MAX)
}
