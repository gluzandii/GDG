use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;
use time::OffsetDateTime;
use utils::jwt::Claims;

#[derive(Serialize)]
struct UserProfile {
    pub email: String,
    pub username: String,
    pub bio: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

fn error_response<S: AsRef<str>>(status: StatusCode, message: S) -> axum::response::Response {
    (status, Json(json!({ "error": message.as_ref() }))).into_response()
}

pub async fn me_route(
    Extension(claims): Extension<Claims>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let user_id = match claims.sub.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            tracing::error!(user_id = claims.sub, "Invalid user ID format");
            return error_response(StatusCode::BAD_REQUEST, "Invalid user ID");
        }
    };

    let user = match sqlx::query_as!(
        UserProfile,
        r#"
        SELECT email, username, bio, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!(user_id = claims.sub, "User not found");
            return error_response(StatusCode::NOT_FOUND, "User not found");
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to fetch user profile");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("A database error occurred: {}", e),
            );
        }
    };

    (StatusCode::OK, Json(user)).into_response()
}
