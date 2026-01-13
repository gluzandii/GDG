//! Common utilities for authentication handlers.
//!
//! Shared functions and error handling for login and registration endpoints.

use api_types::auth::register::LoginAndRegisterResponse;
use axum::Json;
use axum::http::StatusCode;
use axum::http::header::HeaderValue;
use axum::response::IntoResponse;

/// Creates an error response with the specified status code and message.
///
/// # Arguments
///
/// * `status` - The HTTP status code
/// * `message` - The error message to return
///
/// # Returns
///
/// An Axum response with the error details in JSON format.
#[inline(always)]
pub fn error_response(status: StatusCode, message: String) -> axum::response::Response {
    let resp = LoginAndRegisterResponse {
        ok: false,
        message,
        id: None,
    };
    (status, Json(resp)).into_response()
}

/// Creates an authentication cookie with the JWT token.
///
/// # Arguments
///
/// * `user_id` - The user ID to encode in the JWT
///
/// # Returns
///
/// - `Ok(HeaderValue)` - The Set-Cookie header value on success
/// - `Err(Response)` - An error response if JWT generation or cookie building fails
#[inline]
pub fn create_auth_cookie(user_id: i64) -> Result<HeaderValue, axum::response::Response> {
    let jwt_token = utils::jwt::sign_jwt(user_id.to_string()).map_err(|e| {
        tracing::error!(error = ?e, "Failed to sign JWT.");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("An error occurred on our end: {}", e),
        )
    })?;

    utils::jwt::build_cookie(jwt_token).map_err(|e| {
        tracing::error!(error = ?e, "Failed to build cookie.");
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("An error occurred on our end: {}", e),
        )
    })
}
