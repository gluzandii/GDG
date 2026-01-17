use api_types::chats::messages::delete::{
    ApiChatsMessagesDeleteRequest, ApiChatsMessagesDeleteResponse,
};
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use utils::errors::error_response;

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
pub async fn api_chats_messages_delete(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
    Json(payload): Json<ApiChatsMessagesDeleteRequest>,
) -> impl IntoResponse {
    match delete_message_impl(user_id, &pool, payload.conversation_id, payload.message_id).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err((status, message)) => error_response(status, &message),
    }
}

/// Deletes a message within a conversation for an authenticated user.
///
/// Steps:
/// 1. Ensure the user participates in the conversation.
/// 2. Verify the message belongs to the conversation.
/// 3. Delete the message and return confirmation.
pub async fn delete_message_impl(
    user_id: i64,
    pool: &PgPool,
    conversation_id: uuid::Uuid,
    message_id: uuid::Uuid,
) -> Result<ApiChatsMessagesDeleteResponse, (StatusCode, String)> {
    // Validate user participation in the conversation
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

    // Ensure the message exists in the conversation and was sent by the requester
    let message_check = sqlx::query!(
        r#"
        SELECT user_sent_id
        FROM messages
        WHERE id = $1::UUID
          AND conversation_id = $2::UUID
        "#,
        message_id,
        conversation_id
    )
    .fetch_optional(pool)
    .await;

    let message_row = match message_check {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                "Message not found in this conversation.".to_string(),
            ));
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to verify message existence");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while verifying the message.".to_string(),
            ));
        }
    };

    if message_row.user_sent_id != user_id {
        return Err((
            StatusCode::FORBIDDEN,
            "You can only delete messages you sent.".to_string(),
        ));
    }

    // Delete the message
    let delete_result = sqlx::query!(
        r#"
        DELETE FROM messages
        WHERE id = $1::UUID
          AND user_sent_id = $2
        "#,
        message_id,
        user_id
    )
    .execute(pool)
    .await;

    match delete_result {
        Ok(_) => Ok(ApiChatsMessagesDeleteResponse {
            message: "Message deleted successfully.".to_string(),
        }),
        Err(e) => {
            tracing::error!(error = ?e, "Failed to delete message");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while deleting the message.".to_string(),
            ))
        }
    }
}
