//! User profile endpoint handler.
//!
//! Handles fetching the authenticated user's profile information.

use api_types::users::me::MeResponse;
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use utils::{errors::error_response, jwt::Claims};

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
#[tracing::instrument(skip(pool, claims))]
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
        MeResponse,
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
