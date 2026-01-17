use api_types::chats::messages::get::{
    ApiChatsMessagesGetRequest, ApiChatsMessagesGetResponse, ChatItem,
};
use axum::{
    Extension, Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;
use utils::errors::error_response;
use uuid::Uuid;

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
pub async fn api_chats_messages_get(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
    Query(query): Query<ApiChatsMessagesGetRequest>,
) -> impl IntoResponse {
    tracing::debug!(user_id, conversation_id = ?query.conversation_id, "Retrieving messages");

    match get_messages_impl(
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

/// Row structure for chat messages from database.
pub struct ChatRow {
    pub id: Uuid,
    pub content: String,
    pub username: String,
    pub sent_at: time::OffsetDateTime,
}

/// Handles chat message retrieval logic.
///
/// This function:
/// 1. Retrieves messages from a conversation based on query parameters:
///    - Supports cursor-based pagination using `cursor` and `limit`
/// 2. Returns messages in descending order by sent_at timestamp and includes pagination metadata
///
/// # Arguments
///
/// * `user_id` - The authenticated user's ID from the JWT cookie
/// * `pool` - The PostgreSQL connection pool
/// * `conversation_id` - The conversation to retrieve messages from
/// * `cursor` - Optional RFC3339 timestamp for pagination
/// * `limit` - Optional message limit (clamped between 1-100, default 50)
///
/// # Returns
///
/// - `Ok(GetChatsResponse)` with the list of messages on success
/// - `Err((StatusCode, String))` if database operation fails
#[inline(always)]
pub async fn get_messages_impl(
    user_id: i64,
    pool: &PgPool,
    conversation_id: Uuid,
    cursor: Option<String>,
    limit: Option<i64>,
) -> Result<ApiChatsMessagesGetResponse, (StatusCode, String)> {
    // Verify that the user is a participant in the conversation
    let is_participant = sqlx::query!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM conversations
            WHERE id = $1::UUID
              AND (user_id_1 = $2 OR user_id_2 = $2)
        ) as "exists!"
        "#,
        conversation_id,
        user_id
    )
    .fetch_one(pool)
    .await;

    match is_participant {
        Ok(record) if !record.exists => {
            tracing::warn!("User attempted to access conversation they are not part of");
            return Err((
                StatusCode::FORBIDDEN,
                "You are not a participant in this conversation.".to_string(),
            ));
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to verify conversation participation");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while verifying conversation access.".to_string(),
            ));
        }
        _ => {}
    }

    const DEFAULT_LIMIT: i64 = 50;
    const MAX_LIMIT: i64 = 100;

    let limit = limit
        .map(|value| value.clamp(1, MAX_LIMIT))
        .unwrap_or(DEFAULT_LIMIT);
    let fetch_limit = limit + 1;

    let cursor_timestamp = if let Some(cursor) = cursor.as_deref() {
        match time::OffsetDateTime::parse(cursor, &time::format_description::well_known::Rfc3339) {
            Ok(ts) => Some(ts),
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Invalid cursor format. Use RFC3339 timestamp.".to_string(),
                ));
            }
        }
    } else {
        None
    };

    let result = sqlx::query_as!(
        ChatRow,
        r#"
                SELECT messages.id as "id: Uuid", messages.content, users.username, messages.sent_at
        FROM messages
        JOIN users ON messages.user_sent_id = users.id
        WHERE messages.conversation_id = $1::UUID
          AND ($2::TIMESTAMPTZ IS NULL OR messages.sent_at < $2::TIMESTAMPTZ)
        ORDER BY messages.sent_at DESC
        LIMIT $3
        "#,
        conversation_id,
        cursor_timestamp,
        fetch_limit
    )
    .fetch_all(pool)
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
                    id: row.id,
                    content: row.content,
                    user_sent: row.username,
                    sent_at: row
                        .sent_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap_or("Wasn't able to format timestamp".to_string()),
                })
                .collect();

            Ok(ApiChatsMessagesGetResponse {
                chats,
                next_cursor,
                has_more,
            })
        }
        Err(e) => {
            tracing::error!(error = ?e, "An error occurred while retrieving messages");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while retrieving messages.".to_string(),
            ))
        }
    }
}
