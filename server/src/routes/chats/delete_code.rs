use api_types::chats::delete_code::DeleteChatCodeRequest;
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use utils::errors::error_response;

pub async fn delete_code_chat_route(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
    Json(payload): Json<DeleteChatCodeRequest>,
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
        Ok(Some(_)) => (StatusCode::OK).into_response(), // Successfully deleted
        Ok(None) => (StatusCode::NOT_FOUND).into_response(), // Code not found
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
