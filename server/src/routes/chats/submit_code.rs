use api_types::chats::delete_submit_code::DeleteSubmitCodeRequest;
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use utils::errors::error_response;

#[tracing::instrument(name = "Submit a chat code", skip(user_id, pool, payload))]
pub async fn submit_code_chat_route(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
    Json(payload): Json<DeleteSubmitCodeRequest>,
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
        Ok(Some(_)) => {
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
            StatusCode::CREATED.into_response()
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
