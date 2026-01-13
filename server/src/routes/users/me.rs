use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;
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

pub async fn me_route(
    Extension(claims): Extension<Claims>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = sqlx::query_as!(
        UserProfile,
        r#"
        SELECT email, username, bio, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
        claims
            .sub
            .parse::<i64>()
            .map_err(|_| StatusCode::BAD_REQUEST)?
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        tracing::error!(error = ?e, "Failed to fetch user profile");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        tracing::warn!(user_id = claims.sub, "User not found");
        StatusCode::NOT_FOUND
    })?;

    let profile = UserProfile {
        email: user.email,
        username: user.username,
        bio: user.bio,
        created_at: user.created_at,
        updated_at: user.updated_at,
    };

    Ok((StatusCode::OK, Json(profile)))
}
