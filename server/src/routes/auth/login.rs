//! User login endpoint handler.
//!
//! Handles user authentication with password verification
//! and JWT token generation.

use api_types::auth::login::LoginRequest;
use api_types::auth::register::LoginAndRegisterResponse;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use axum::response::IntoResponse;
use sqlx::PgPool;
use sqlx::prelude::FromRow;
use utils::hashing;

use crate::routes::auth::register::error_response;

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
pub async fn login(State(pool): State<PgPool>, Json(req): Json<LoginRequest>) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        tracing::info!(error = ?e, "Validation failed");
        return error_response(
            StatusCode::BAD_REQUEST,
            format!("Your request was invalid: {}", e),
        );
    }

    let LoginRequest {
        person,
        password,
        is_email,
    } = req;

    // Query user based on whether it's email or username
    let user = if is_email {
        sqlx::query_as::<_, UserRecord>(
            r#"
            SELECT id, password_hash
            FROM users
            WHERE email = $1
            "#,
        )
    } else {
        sqlx::query_as::<_, UserRecord>(
            r#"
            SELECT id, password_hash
            FROM users
            WHERE username = $1
            "#,
        )
    }
    .bind(&person)
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

    // Generate JWT token
    let jwt_token = match utils::jwt::sign_jwt(user.id.to_string()) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to sign JWT for user.");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("An error occurred on our end: {}", e),
            );
        }
    };

    // Build cookie
    let cookie = match utils::jwt::build_cookie(jwt_token) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to build cookie for user.");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("An error occurred on our end: {}", e),
            );
        }
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
