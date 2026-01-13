//! User registration endpoint handler.
//!
//! Handles the creation of new user accounts with validation,
//! password hashing, and JWT token generation.

use super::common::{create_auth_cookie, error_response};
use api_types::auth::register::{LoginAndRegisterResponse, RegisterRequest};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::SET_COOKIE;
use axum::response::IntoResponse;
use sqlx::PgPool;
use utils::hashing;

/// Handles user registration requests.
///
/// This endpoint:
/// 1. Validates the registration request (email format, password complexity)
/// 2. Checks if the username or email already exists
/// 3. Hashes the password using Argon2
/// 4. Inserts the new user into the database
/// 5. Generates a JWT token for the new user
/// 6. Sets a session cookie with the JWT token
///
/// # Arguments
///
/// * `pool` - The PostgreSQL connection pool
/// * `req` - The registration request containing username, email, and password
///
/// # Returns
///
/// - `201 CREATED` with user details and session cookie on success
/// - `401 UNAUTHORIZED` if validation fails
/// - `409 CONFLICT` if username or email already exists
/// - `500 INTERNAL SERVER ERROR` if any server-side operation fails
///
/// # Example Request
///
/// ```json
/// {
///   "username": "john_doe",
///   "email": "john@example.com",
///   "password": "SecurePass123"
/// }
/// ```
#[tracing::instrument(skip(pool, req))]
pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        tracing::error!(error = ?e, "Validation failed");
        return error_response(
            StatusCode::UNAUTHORIZED,
            format!("Your request was invalid: {}", e),
        );
    }

    let RegisterRequest {
        username,
        email,
        password,
        bio,
    } = req;

    // Check if username or email already exists
    let existing = match sqlx::query!(
        r#"
        SELECT
            EXISTS(SELECT 1 FROM users WHERE username = $1) as "username_exists!",
            EXISTS(SELECT 1 FROM users WHERE email = $2) as "email_exists!"
        "#,
        username,
        email
    )
    .fetch_one(&pool)
    .await
    {
        Ok(record) => record,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to query existing users. Error occurred while querying database.");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("A database error occurred on our end: {}", e),
            );
        }
    };

    if existing.username_exists && existing.email_exists {
        tracing::debug!(
            username,
            email,
            "Attempt to register with existing username and email",
        );
        tracing::info!("User registration failed: username and email already exist");
        return error_response(
            StatusCode::CONFLICT,
            "This user already exists.".to_string(),
        );
    }
    if existing.username_exists {
        tracing::debug!(username, "Attempt to register with existing username",);
        tracing::info!("User registration failed: username already exists");
        return error_response(StatusCode::CONFLICT, "Username already exists".to_string());
    }
    if existing.email_exists {
        tracing::debug!(email, "Attempt to register with existing email",);
        tracing::info!("User registration failed: email already exists");
        return error_response(StatusCode::CONFLICT, "Email already exists".to_string());
    }

    let hashed = match hashing::hash_password(password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to hash password, for registering a user.");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("An error occurred on our end: {}", e),
            );
        }
    };

    let user = match sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash, bio)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        username,
        email,
        hashed,
        bio
    )
    .fetch_one(&pool)
    .await
    {
        Ok(record) => record,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to insert new user.");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("A database error occurred on our end: {}", e),
            );
        }
    };

    let cookie = match create_auth_cookie(user.id) {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    tracing::debug!("Setting session cookie for new user.");

    let resp = LoginAndRegisterResponse {
        ok: true,
        message: "User successfully created.".to_string(),
        id: Some(user.id),
    };
    let mut resp = (StatusCode::CREATED, Json(resp)).into_response();
    resp.headers_mut().insert(SET_COOKIE, cookie);

    resp
}
