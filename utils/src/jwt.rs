//! JWT token generation and verification.
//!
//! This module provides utilities for creating and verifying JWT tokens,
//! as well as building secure HTTP cookies for session management.

use axum::http::HeaderValue;
use axum::http::header::InvalidHeaderValue;
use cookie::Cookie;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};
use serde::{Deserialize, Serialize};
use std::env;

/// JWT claims structure.
///
/// Contains the standard JWT claims for authentication tokens.
#[derive(Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at time (Unix timestamp)
    pub iat: usize,
    /// Expiration time (Unix timestamp)
    pub exp: usize,
}

/// Retrieves the JWT secret key from environment variables.
///
/// # Returns
///
/// - `Ok(String)` containing the secret key
/// - `Err(jsonwebtoken::errors::Error)` if the `JWT_SECRET_KEY` environment variable is not set
fn get_secret_key() -> Result<String, jsonwebtoken::errors::Error> {
    match env::var("JWT_SECRET_KEY") {
        Ok(val) => Ok(val),
        Err(e) => {
            tracing::error!(error = ?e, "JWT_SECRET_KEY environment variable not set");
            Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
            ))
        }
    }
}

/// Creates a signed JWT token for a user.
///
/// The token is valid for 7 days and uses the HS256 algorithm.
///
/// # Arguments
///
/// * `user_id` - The user's unique identifier
///
/// # Returns
///
/// - `Ok(String)` containing the signed JWT token
/// - `Err(jsonwebtoken::errors::Error)` if signing fails or the secret key is not set
///
/// # Example
///
/// ```ignore
/// let token = sign_jwt("12345")?;
/// ```
pub fn sign_jwt<S: AsRef<str>>(user_id: S) -> Result<String, jsonwebtoken::errors::Error> {
    tracing::trace!("Signing JWT");

    let secret = get_secret_key()?;
    let iat = get_current_timestamp() as usize;
    let exp = iat + (7 * 24 * 60 * 60); // 1 week

    let claims = Claims {
        sub: user_id.as_ref().to_string(),
        iat,
        exp,
    };

    let header = Header::new(Algorithm::HS256);

    tracing::trace!("Signing JWT");
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Verifies and decodes a JWT token.
///
/// # Arguments
///
/// * `token` - The JWT token string to verify
///
/// # Returns
///
/// - `Ok(Claims)` containing the decoded claims if the token is valid
/// - `Err(jsonwebtoken::errors::Error)` if verification fails, the token is expired, or the secret key is not set
///
/// # Example
///
/// ```ignore
/// let claims = verify_jwt(&token)?;
/// println!("User ID: {}", claims.sub);
/// ```
pub fn verify_jwt<S: AsRef<str>>(token: S) -> Result<Claims, jsonwebtoken::errors::Error> {
    tracing::trace!("Verifying JWT");

    let secret = get_secret_key()?;
    let validation = Validation::new(Algorithm::HS256);

    let data = decode::<Claims>(
        token.as_ref(),
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    tracing::trace!("JWT verified");
    Ok(data.claims)
}

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
