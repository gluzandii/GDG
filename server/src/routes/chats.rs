//! Chat route handlers.
//!
//! This module contains all chat-related endpoints including creation,
//! deletion, and real-time WebSocket communication.

use api_types::chats::{ChatItem, GetChatsQuery, GetChatsResponse};

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

/// Row structure for chat messages from database.
struct ChatRow {
    content: String,
    username: String,
    sent_at: time::OffsetDateTime,
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

    // Verify that the user is a participant in the conversation
    let is_participant = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM conversations
            WHERE id = $1::UUID
              AND (user_id_1 = $2 OR user_id_2 = $2)
        ) as "exists!"
        "#,
        query.conversation_id,
        user_id
    )
    .fetch_one(&pool)
    .await;

    match is_participant {
        Ok(record) if !record.exists => {
            tracing::warn!("User attempted to access conversation they are not part of");
            return error_response(
                StatusCode::FORBIDDEN,
                "You are not a participant in this conversation.",
            );
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to verify conversation participation");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while verifying conversation access.",
            );
        }
        _ => {}
    }

    const DEFAULT_LIMIT: i64 = 50;
    const MAX_LIMIT: i64 = 100;

    let limit = query
        .limit
        .map(|value| value.clamp(1, MAX_LIMIT))
        .unwrap_or(DEFAULT_LIMIT);
    let fetch_limit = limit + 1;

    let cursor_timestamp = if let Some(cursor) = query.cursor.as_deref() {
        match time::OffsetDateTime::parse(cursor, &time::format_description::well_known::Rfc3339) {
            Ok(ts) => Some(ts),
            Err(_) => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    "Invalid cursor format. Use RFC3339 timestamp.",
                );
            }
        }
    } else {
        None
    };

    let result = sqlx::query_as!(
        ChatRow,
        r#"
        SELECT messages.content, users.username, messages.sent_at
        FROM messages
        JOIN users ON messages.user_sent_id = users.id
        WHERE messages.conversation_id = $1::UUID
          AND ($2::TIMESTAMPTZ IS NULL OR messages.sent_at < $2::TIMESTAMPTZ)
        ORDER BY messages.sent_at DESC
        LIMIT $3
        "#,
        query.conversation_id,
        cursor_timestamp,
        fetch_limit
    )
    .fetch_all(&pool)
    .await;

    match result {
        Ok(mut rows) => {
            let has_more = (rows.len() as i64) > limit;
            if has_more {
                rows.truncate(limit as usize);
            }

            let next_cursor = rows.last().and_then(|row| {
                row.sent_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .ok()
            });

            let chats: Vec<ChatItem> = rows
                .into_iter()
                .map(|row| ChatItem {
                    content: row.content,
                    user_sent: row.username,
                    sent_at: row
                        .sent_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap_or("Wasn't able to format timestamp".to_string()),
                })
                .collect();

            (
                StatusCode::OK,
                Json(GetChatsResponse {
                    chats,
                    next_cursor,
                    has_more,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = ?e, "An error occurred while retrieving messages");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while retrieving messages.",
            )
        }
    }
}
