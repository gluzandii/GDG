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
use futures::StreamExt;
use sqlx::PgPool;
use utils::errors::error_response;

pub async fn ws_handler(
    Query(params): Query<ChatQuery>,
    Extension(user_id): Extension<i64>,
    ws: WebSocketUpgrade,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let chat_code = match params.chat_code {
        Some(code) => code,
        None => return error_response(StatusCode::BAD_REQUEST, "Chat code not provided"),
    };

    let is_participant = match sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM conversations
            WHERE id = $1 AND (user_id_1 = $2 OR user_id_2 = $2)
        )
        "#,
        chat_code,
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
        if let Err(e) = handle_socket(socket, pool, chat_code, user_id).await {
            tracing::error!("WebSocket session ended with error: {}", e);
        }
    })
}

async fn handle_socket(
    mut socket: WebSocket,
    pool: PgPool,
    conversation_id: uuid::Uuid,
    user_id: i64,
) -> Result<(), String> {
    while let Some(msg_result) = socket.recv().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                let content = text.trim();
                if content.is_empty() {
                    continue;
                }

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
                    return Err("Failed to persist message".into());
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(e) => {
                return Err(format!("WebSocket error: {}", e));
            }
        }
    }

    Ok(())
}
