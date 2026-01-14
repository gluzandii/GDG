use api_types::chats::delete_submit_code::DeleteSubmitCodeRequest;
use axum::{Extension, Json, extract::State, response::IntoResponse};
use sqlx::PgPool;

#[tracing::instrument(name = "Submit a chat code", skip(user_id, pool, payload))]
pub async fn submit_code_chat_route(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
    Json(payload): Json<DeleteSubmitCodeRequest>,
) -> impl IntoResponse {
    // Implementation for submitting a chat code goes here
}
