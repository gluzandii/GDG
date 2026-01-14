use axum::http::{HeaderValue, StatusCode, header::InvalidHeaderValue};
use cookie::Cookie;

use crate::errors;

/// Builds an HTTP cookie for session management.
///
/// Creates a secure HTTP-only cookie named `session_token` with the following properties:
/// - Path: `/`
/// - HTTP-only: true (not accessible via JavaScript)
/// - Secure: false (set to true in production with HTTPS)
/// - SameSite: Lax
/// - Max-Age: 7 days
///
/// # Arguments
///
/// * `value` - The JWT token to store in the cookie
///
/// # Returns
///
/// - `Ok(HeaderValue)` containing the formatted cookie header
/// - `Err(InvalidHeaderValue)` if the cookie string contains invalid characters
///
/// # Example
///
/// ```ignore
/// let cookie = build_cookie(jwt_token)?;
/// response.headers_mut().insert(SET_COOKIE, cookie);
/// ```
pub fn build_cookie<S: Into<String>>(value: S) -> Result<HeaderValue, InvalidHeaderValue> {
    let cookie = Cookie::build(("session_token", value.into()))
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(cookie::SameSite::Lax)
        .max_age(time::Duration::days(7))
        .build();
    cookie.to_string().parse()
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
    let jwt_token = crate::jwt::sign_jwt(user_id.to_string()).map_err(|e| {
        tracing::error!(error = ?e, "Failed to sign JWT.");
        errors::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("An error occurred on our end: {}", e),
        )
    })?;

    build_cookie(jwt_token).map_err(|e| {
        tracing::error!(error = ?e, "Failed to build cookie.");
        errors::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("An error occurred on our end: {}", e),
        )
    })
}
