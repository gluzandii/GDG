//! WebSocket handler for real-time chat functionality.
//!
//! This module implements a real-time chat system using WebSockets and PostgreSQL LISTEN/NOTIFY.
//! Messages are persisted to the database and broadcast to connected clients in real-time.

use api_types::chats::ws::ChatQuery;
use axum::Extension;
use axum::http::StatusCode;
use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgListener};
use utils::errors::error_response;

/// Represents a message notification payload from PostgreSQL LISTEN/NOTIFY.
#[derive(Serialize, Deserialize)]
struct MessageNotification {
    /// ID of the user who sent the message
    user_id: i64,
    /// Content of the message
    content: String,
}

/// Handles WebSocket upgrades for real-time chat.
///
/// Validates that the user is a participant in the conversation, then upgrades
/// the HTTP connection to a WebSocket and delegates to `handle_socket`.
///
/// # Arguments
/// * `params` - Query parameters containing the chat ID
/// * `user_id` - The authenticated user ID from the JWT extension
/// * `ws` - WebSocket upgrade handler
/// * `pool` - PostgreSQL connection pool
///
/// # Returns
/// Either an error response (if validation fails) or a WebSocket upgrade response
#[tracing::instrument(skip(ws, pool, user_id, params))]
pub async fn ws_handler(
    Query(params): Query<ChatQuery>,
    Extension(user_id): Extension<i64>,
    ws: WebSocketUpgrade,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let chat_id = match params.chat_id {
        Some(id) => id,
        None => return error_response(StatusCode::BAD_REQUEST, "Chat ID not provided"),
    };

    let is_participant = match sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM conversations
            WHERE id = $1 AND (user_id_1 = $2 OR user_id_2 = $2)
        )
        "#,
        chat_id,
        user_id
    )
    .fetch_one(&pool)
    .await
    {
        Ok(res) => res.unwrap_or(false),
        Err(e) => {
            tracing::error!("Failed to verify conversation participant: {}", e);
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to verify conversation participant",
            );
        }
    };

    if !is_participant {
        return error_response(
            StatusCode::UNAUTHORIZED,
            "Not authorized for this conversation",
        );
    }

    ws.on_upgrade(move |socket| async move {
        handle_socket(socket, pool, chat_id, user_id).await;
    })
}

#[tracing::instrument(skip(socket, pool, user_id, conversation_id))]
async fn handle_socket(
    mut socket: WebSocket,
    pool: PgPool,
    conversation_id: uuid::Uuid,
    user_id: i64,
) {
    // Create a PostgreSQL listener for this conversation
    let mut listener = match PgListener::connect_with(&pool).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to create PgListener: {}", e);
            return;
        }
    };

    let channel = format!("conversation_{}", conversation_id);
    if let Err(e) = listener.listen(&channel).await {
        tracing::error!("Failed to listen to channel {}: {}", channel, e);
        return;
    }

    let mut notification_stream = listener.into_stream();

    loop {
        tokio::select! {
            // Handle incoming WebSocket messages from the client
            msg_result = socket.recv() => {
                match msg_result {
                    Some(Ok(Message::Text(text))) => {
                        let content = text.trim();
                        if content.is_empty() {
                            continue;
                        }

                        // Insert message into database (trigger will send notification)
                        if let Err(e) = sqlx::query!(
                            r#"
                            INSERT INTO messages (conversation_id, user_sent_id, content)
                            VALUES ($1, $2, $3)
                            "#,
                            conversation_id,
                            user_id,
                            content
                        )
                        .execute(&pool)
                        .await
                        {
                            tracing::error!("Failed to persist message: {}", e);
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => continue,
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }

            // Handle incoming PostgreSQL notifications
            notification = notification_stream.next() => {
                match notification {
                    Some(Ok(notification)) => {
                        // Parse the notification payload
                        match serde_json::from_str::<MessageNotification>(notification.payload()) {
                            Ok(msg_notif) => {
                                // Don't send the message back to the sender
                                if msg_notif.user_id != user_id {
                                    if let Err(e) = socket.send(Message::Text(msg_notif.content.into())).await {
                                        tracing::error!("Failed to send message to WebSocket: {}", e);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to parse notification payload: {}", e);
                            }
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!("Notification stream error: {}", e);
                        break;
                    }
                    None => break,
                }
            }
        }
    }
}
