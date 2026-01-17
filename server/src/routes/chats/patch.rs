use api_types::chats::UpdateMessageResponse;
use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

/// Updates a message within a conversation for an authenticated user.
///
/// Steps:
/// 1. Ensure the user participates in the conversation.
/// 2. Verify the message belongs to the conversation and was sent by the user.
/// 3. Update the message content and edited_at timestamp.
pub async fn update_message_impl(
    user_id: i64,
    pool: &PgPool,
    conversation_id: Uuid,
    message_id: Uuid,
    content: String,
) -> Result<UpdateMessageResponse, (StatusCode, String)> {
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
            "You can only update messages you sent.".to_string(),
        ));
    }

    // Update the message content and edited_at timestamp
    let update_result = sqlx::query!(
        r#"
        UPDATE messages
        SET content = $1, edited_at = CURRENT_TIMESTAMP
        WHERE id = $2::UUID
          AND user_sent_id = $3
        RETURNING edited_at
        "#,
        content,
        message_id,
        user_id
    )
    .fetch_optional(pool)
    .await;

    match update_result {
        Ok(Some(row)) => {
            let edited_at = row
                .edited_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or("Wasn't able to format timestamp".to_string());

            Ok(UpdateMessageResponse {
                message: "Message updated successfully.".to_string(),
                edited_at,
            })
        }
        Ok(None) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to update message.".to_string(),
        )),
        Err(e) => {
            tracing::error!(error = ?e, "Failed to update message");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "An error occurred while updating the message.".to_string(),
            ))
        }
    }
}
