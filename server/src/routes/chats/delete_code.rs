//! Delete chat code endpoint handler.
//!
//! Handles deletion of chat codes for the authenticated user.

use api_types::chats::delete_submit_code::{DeleteSubmitCodeRequest, DeleteSubmitCodeResponse};
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use utils::errors::error_response;

/// Handles chat code deletion requests.
///
/// This endpoint:
/// 1. Extracts the user ID from the authentication cookie
/// 2. Validates that the chat code exists and is owned by the user
/// 3. Deletes the chat code from the database
/// 4. Returns the associated conversation ID if available
///
/// # Arguments
///
/// * `user_id` - The authenticated user's ID from the JWT cookie
/// * `pool` - The PostgreSQL connection pool
/// * `payload` - The delete request containing the chat code
///
/// # Returns
///
/// - `200 OK` with deletion confirmation
/// - `404 NOT FOUND` if the chat code doesn't exist or isn't owned by the user
/// - `500 INTERNAL SERVER ERROR` if database operation fails
#[tracing::instrument(name = "Delete a chat code", skip(pool, user_id, payload))]
pub async fn delete_code_chat_route(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
    Json(payload): Json<DeleteSubmitCodeRequest>,
) -> impl IntoResponse {
    // Check if the chat code exists and delete it
    let result = sqlx::query!(
        "DELETE FROM chat_codes WHERE code = $1 AND user_id = $2 RETURNING id",
        payload.code as i32,
        user_id
    )
    .fetch_optional(&pool)
    .await;

    match result {
        Ok(Some(_)) => (
            StatusCode::OK,
            Json(DeleteSubmitCodeResponse {
                conversation_id: None,
                message: "Chat code deleted successfully".to_string(),
            }),
        )
            .into_response(), // Successfully deleted
        Ok(None) => error_response(StatusCode::NOT_FOUND, "Chat code not found."), // Code not found
        Err(e) => {
            tracing::error!(
                error = ?e,
                "An error occurred while trying to delete code: {}",
                payload.code
            );
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred on our end while trying to delete the chat code.",
            )
        } // Database error
    }
}
