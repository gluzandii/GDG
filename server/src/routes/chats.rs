//! Chat route handlers.
//!
//! This module contains all chat-related endpoints including creation,
//! deletion, and real-time WebSocket communication.

use api_types::chats::{DeleteMessageRequest, GetChatsQuery, UpdateMessageRequest};

use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;
use utils::errors::error_response;

/// Create new chat endpoint handler.
pub mod new_code;

/// Delete chat endpoint handler.
pub mod delete_code;

/// Submit chat code endpoint handler.
pub mod submit_code;

/// WebSocket real-time chat handler.
pub mod ws;

/// Delete message implementation.
pub mod delete;

/// Get messages implementation.
pub mod get;

/// Update message implementation.
pub mod patch;

/// Deletes a message within a conversation for an authenticated user.
///
/// Steps:
/// 1. Ensure the user participates in the conversation.
/// 2. Verify the message belongs to the conversation.
/// 3. Delete the message and return confirmation.
#[tracing::instrument(
    skip(pool, user_id),
    fields(conversation_id = ?payload.conversation_id, message_id = ?payload.message_id)
)]
pub async fn delete_chat_message_route(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
    Json(payload): Json<DeleteMessageRequest>,
) -> impl IntoResponse {
    match delete::delete_message_impl(user_id, &pool, payload.conversation_id, payload.message_id)
        .await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err((status, message)) => error_response(status, &message),
    }
}

/// Updates a message within a conversation for an authenticated user.
///
/// Steps:
/// 1. Ensure the user participates in the conversation.
/// 2. Verify the message belongs to the conversation and was sent by the user.
/// 3. Update the message content and edited_at timestamp.
#[tracing::instrument(
    skip(pool, user_id),
    fields(conversation_id = ?payload.conversation_id, message_id = ?payload.message_id)
)]
pub async fn update_chat_message_route(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
    Json(payload): Json<UpdateMessageRequest>,
) -> impl IntoResponse {
    match patch::update_message_impl(
        user_id,
        &pool,
        payload.conversation_id,
        payload.message_id,
        payload.content,
    )
    .await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err((status, message)) => error_response(status, &message),
    }
}

/// Handles chat message retrieval requests.
///
/// This endpoint:
/// 1. Extracts the user ID from the authentication cookie
/// 2. Retrieves messages from a conversation based on query parameters:
///    - Supports cursor-based pagination using `cursor` and `limit`
/// 3. Returns messages in descending order by sent_at timestamp and includes pagination metadata
///
/// # Arguments
///
/// * `user_id` - The authenticated user's ID from the JWT cookie
/// * `pool` - The PostgreSQL connection pool
/// * `query` - Query parameters including conversation_id and optional filters
///
/// # Returns
///
/// - `200 OK` with the list of messages on success
/// - `500 INTERNAL SERVER ERROR` if database operation fails
#[tracing::instrument(skip(pool, user_id), fields(cursor = ?query.cursor, limit = ?query.limit))]
pub async fn get_chats_route(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
    Query(query): Query<GetChatsQuery>,
) -> impl IntoResponse {
    tracing::debug!(user_id, conversation_id = ?query.conversation_id, "Retrieving messages");

    match get::get_messages_impl(
        user_id,
        &pool,
        query.conversation_id,
        query.cursor,
        query.limit,
    )
    .await
    {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err((status, message)) => error_response(status, &message),
    }
}
