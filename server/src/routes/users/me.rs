use axum::{Extension, extract::State, response::IntoResponse};
use sqlx::PgPool;
use utils::jwt::Claims;

pub async fn me_route(
    Extension(claims): Extension<Claims>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    "hi"
}
