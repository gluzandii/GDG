//! User login endpoint handler.
//!
//! Handles user authentication with password verification
//! and JWT token generation.

use api_types::auth::login::AuthLoginRequest;
use api_types::auth::register::LoginAndRegisterResponse;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use axum::response::IntoResponse;
use sqlx::PgPool;
use sqlx::prelude::FromRow;
use utils::cookies::create_auth_cookie;
use utils::errors::error_response;
use utils::hashing;

#[derive(FromRow)]
struct UserRecord {
    id: i64,
    password_hash: String,
}

/// Handles user login requests.
///
/// This endpoint:
/// 1. Validates the login request (username/email and password not empty)
/// 2. Queries the database for a user with the provided username or email
/// 3. Verifies the password against the stored hash
/// 4. Generates a JWT token for the authenticated user
/// 5. Sets a session cookie with the JWT token
///
/// # Arguments
///
/// * `pool` - The PostgreSQL connection pool
/// * `req` - The login request containing person and password
///
/// # Returns
///
/// - `200 OK` with user details and session cookie on success
/// - `400 BAD REQUEST` if validation fails
/// - `401 UNAUTHORIZED` if credentials are invalid
/// - `500 INTERNAL SERVER ERROR` if any server-side operation fails
///
/// # Example Request
///
/// ```json
/// {
///   "person": "john_doe",
///   "password": "SecurePass123",
///   "is_email": false
/// }
/// ```
#[tracing::instrument(skip(pool, req))]
pub async fn login_route(
    State(pool): State<PgPool>,
    Json(req): Json<AuthLoginRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        tracing::info!(error = ?e, "Validation failed");
        return error_response(
            StatusCode::BAD_REQUEST,
            format!("Your request was invalid: {}", e),
        );
    }

    let AuthLoginRequest {
        person,
        password,
        is_email,
    } = req;

    // Query user based on whether it's email or username
    let user = sqlx::query_as!(
        UserRecord,
        r#"
        SELECT id, password_hash
        FROM users
        WHERE (email = $1 AND $2) OR (username = $1 AND NOT $2)
        "#,
        person,
        is_email,
    )
    .fetch_optional(&pool)
    .await;

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::info!(person, is_email, "Login attempt with non-existent user");
            return error_response(StatusCode::UNAUTHORIZED, "Invalid credentials".to_string());
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to query user from database.");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("A database error occurred on our end: {}", e),
            );
        }
    };

    // Verify password
    match hashing::verify_password(&password, &user.password_hash) {
        Ok(true) => {
            tracing::info!(user_id = user.id, "Password verification successful");
        }
        Ok(false) => {
            tracing::info!(user_id = user.id, "Login attempt with invalid password");
            return error_response(StatusCode::UNAUTHORIZED, "Invalid credentials".to_string());
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to verify password.");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("An error occurred on our end: {}", e),
            );
        }
    }

    // Generate JWT token and cookie
    let cookie = match create_auth_cookie(user.id) {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    tracing::debug!("Setting session cookie for user.");

    let resp = LoginAndRegisterResponse {
        ok: true,
        message: "Login successful".to_string(),
        id: Some(user.id),
    };
    let mut resp = (StatusCode::OK, Json(resp)).into_response();
    resp.headers_mut().insert(SET_COOKIE, cookie);

    resp
}
