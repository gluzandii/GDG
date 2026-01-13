//! Authentication middleware for protected routes.
//!
//! This middleware validates JWT tokens from cookies and prevents
//! unauthorized access to protected endpoints.

use axum::body::Body;
use axum::http::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use tower_cookies::Cookies;

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
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    // Extract the auth token from cookie
    let token = cookies
        .get("auth_token")
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

    // Store claims in request extensions so handlers can access it
    req.extensions_mut().insert(claims);

    tracing::debug!("Auth middleware passed");
    Ok(next.run(req).await)
}
