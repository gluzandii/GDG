//! User registration request and response types.

use serde::{Deserialize, Serialize};

use crate::auth::EMAIL_REGEX;

/// Request payload for user registration.
///
/// Contains the username, email, and password for a new user account.
#[derive(Deserialize)]
pub struct AuthRegisterRequest {
    /// The desired username for the new account.
    pub username: String,
    /// The email address for the new account.
    pub email: String,
    /// The password for the new account (will be hashed before storage).
    pub password: String,
    /// The user's bio.
    pub bio: Option<String>,
}

/// Response payload for user registration.
///
/// Indicates whether registration succeeded and provides relevant information.
#[derive(Serialize)]
pub struct LoginAndRegisterResponse {
    /// Whether the registration was successful.
    pub ok: bool,
    /// A human-readable message describing the result.
    pub message: String,
    /// The ID of the newly created user (only present if registration succeeded).
    pub id: Option<i64>,
}

impl AuthRegisterRequest {
    /// Validates the registration request.
    ///
    /// Checks that:
    /// - The email is in a valid format
    /// - The password is at least 6 characters long
    /// - The password contains at least one uppercase letter, one lowercase letter, and one digit
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all validation passes
    /// - `Err(String)` with a descriptive error message if validation fails
    pub fn validate(&self) -> Result<(), String> {
        if !EMAIL_REGEX.is_match(&self.email) {
            tracing::debug!("Invalid email address");
            return Err("Email format is invalid".into());
        }

        utils::hashing::is_password_suitable(&self.password)
    }
}
