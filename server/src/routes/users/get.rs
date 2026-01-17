//! User profile endpoint handler.
//!
//! Handles fetching the authenticated user's profile information.

use api_types::users::get::UsersMeResponse;
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use utils::errors::error_response;

/// Handles fetching the authenticated user's profile.
///
/// This endpoint retrieves the profile information for the currently
/// authenticated user based on the JWT claims extracted from the session cookie.
///
/// # Arguments
///
/// * `claims` - JWT claims containing the authenticated user's ID
/// * `pool` - The PostgreSQL connection pool
///
/// # Returns
///
/// - `200 OK` with user profile data (email, username, bio, timestamps)
/// - `400 BAD REQUEST` if the user ID in the JWT is invalid
/// - `404 NOT FOUND` if the user doesn't exist in the database
/// - `500 INTERNAL SERVER ERROR` if a database error occurs
///
/// # Example Response
///
/// ```json
/// {
///   "email": "john@example.com",
///   "username": "john_doe",
///   "bio": "Software developer",
///   "created_at": "2026-01-14T10:30:00Z",
///   "updated_at": "2026-01-14T10:30:00Z"
/// }
/// ```
#[tracing::instrument(skip(pool, user_id))]
pub async fn api_users_get(
    Extension(user_id): Extension<i64>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let user = match sqlx::query_as!(
        UsersMeResponse,
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
            tracing::warn!(user_id, "User not found");
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
