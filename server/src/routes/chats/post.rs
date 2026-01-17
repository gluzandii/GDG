//! Submit chat code endpoint handler.
//!
//! Handles the submission of chat codes to establish conversations between users.

use api_types::chats::codes::post::{ApiChatsCodesPostRequest, ApiChatsCodesPostResponse};
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use utils::errors::error_response;

/// Handles chat code submission requests.
///
/// This endpoint:
/// 1. Validates that the chat code exists and is owned by another user
/// 2. Checks if a conversation already exists between the two users
/// 3. Creates a new conversation if one doesn't exist
/// 4. Returns the conversation ID
///
/// # Arguments
///
/// * `user_id` - The authenticated user's ID from the JWT cookie
/// * `pool` - The PostgreSQL connection pool
/// * `payload` - The submit request containing the chat code
///
/// # Returns
///
/// - `200 OK` with the conversation ID
/// - `400 BAD REQUEST` if trying to start a conversation with yourself
/// - `404 NOT FOUND` if the chat code doesn't exist
/// - `500 INTERNAL SERVER ERROR` if database operations fail
#[tracing::instrument(name = "Submit a chat code", skip(user_id, pool, payload))]
pub async fn api_chats_post(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
    Json(payload): Json<ApiChatsCodesPostRequest>,
) -> impl IntoResponse {
    tracing::debug!(user_id, code = payload.code, "Submitting chat code");

    // Verify the code exists and fetch its owner
    let owner = sqlx::query!(
        "SELECT user_id FROM chat_codes WHERE code = $1",
        payload.code as i32
    )
    .fetch_optional(&pool)
    .await;

    let target_user_id = match owner {
        Ok(Some(row)) => row.user_id,
        Ok(None) => return error_response(StatusCode::NOT_FOUND, "Chat code not found."),
        Err(e) => {
            tracing::error!(error = ?e, user_id, code = payload.code, "Failed to look up chat code");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while looking up the chat code.",
            );
        }
    };

    if target_user_id == user_id {
        return error_response(
            StatusCode::BAD_REQUEST,
            "You cannot start a conversation with yourself.",
        );
    }

    // Attempt to create the conversation if it doesn't already exist
    let insert_result = sqlx::query!(
        r#"
        INSERT INTO conversations (user_id_1, user_id_2)
        VALUES (LEAST($1, $2)::BIGINT, GREATEST($1, $2)::BIGINT)
        ON CONFLICT (user_id_1, user_id_2) DO NOTHING
        RETURNING id
        "#,
        target_user_id.to_string(),
        user_id.to_string(),
    )
    .fetch_optional(&pool)
    .await;

    match insert_result {
        Ok(Some(uid)) => {
            // Delete the chat code after successful conversation creation
            if let Err(e) = sqlx::query!(
                "DELETE FROM chat_codes WHERE code = $1",
                payload.code as i32
            )
            .execute(&pool)
            .await
            {
                tracing::warn!(error = ?e, code = payload.code, "Failed to delete chat code");
            }
            (
                StatusCode::CREATED,
                Json(ApiChatsCodesPostResponse {
                    conversation_id: Some(uid.id),
                    message: "Conversation created successfully".to_string(),
                }),
            )
                .into_response()
        }
        Ok(None) => error_response(StatusCode::CONFLICT, "Conversation already exists."),
        Err(e) => {
            tracing::error!(
                error = ?e,
                user_id,
                target_user_id,
                code = payload.code,
                "Failed to create conversation"
            );
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while creating the conversation.",
            )
        }
    }
}
