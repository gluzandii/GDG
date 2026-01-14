//! Authentication middleware for protected routes.
//!
//! This middleware validates JWT tokens from cookies and prevents
//! unauthorized access to protected endpoints.

use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;

/// Authentication middleware that validates JWT tokens from cookies.
///
/// This middleware:
/// 1. Extracts the `auth_token` cookie from the request
/// 2. Decodes and validates the JWT token
/// 3. Stores the claims in request extensions for handler access
/// 4. Returns 401 Unauthorized if the token is missing or invalid
///
/// # Example
///
/// ```rust,no_run
/// use axum::{Router, middleware};
/// use middleware::auth::auth_middleware;
///
/// let app = Router::new()
///     .route("/protected", get(protected_handler))
///     .layer(middleware::from_fn(auth_middleware));
/// ```
pub async fn auth_middleware(
    cookies: CookieJar,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    // Extract the auth token from cookie
    let token = cookies
        .get("session_token")
        .map(|c| c.value().to_string())
        .ok_or_else(|| {
            tracing::warn!("No auth_token cookie found");
            StatusCode::UNAUTHORIZED
        })?;

    // Decode and validate the JWT
    let claims = utils::jwt::verify_jwt(&token).map_err(|e| {
        tracing::warn!(error = ?e, "JWT decode failed, unauthorized.");
        StatusCode::UNAUTHORIZED
    })?;
    let uid = claims.sub.parse::<i64>().map_err(|e| {
        tracing::warn!(error = ?e, "Invalid user ID in JWT claims cookie.");
        StatusCode::BAD_REQUEST
    })?;

    // Store claims in request extensions so handlers can access it
    req.extensions_mut().insert(uid);

    tracing::debug!("Auth middleware passed");
    Ok(next.run(req).await)
}
