//! User login request and response types.

use serde::Deserialize;

use crate::auth::EMAIL_REGEX;

/// Request payload for user login.
///
/// Contains the username or email and password for authentication.
#[derive(Deserialize)]
pub struct LoginRequest {
    /// The username or email address of the user.
    pub person: String,
    /// The password for authentication.
    pub password: String,

    /// Indicates whether the username_or_email field is an email address.
    pub is_email: bool,
}

impl LoginRequest {
    /// Validates the login request.
    ///
    /// Checks that:
    /// - The username_or_email field is not empty
    /// - The password field is not empty
    ///
    /// # Returns
    ///
    /// - `Ok(())` if all validation passes
    /// - `Err(String)` with a descriptive error message if validation fails
    pub fn validate(&self) -> Result<(), String> {
        if self.person.trim().is_empty() {
            return Err("Username or email is required".to_string());
        }

        if self.is_email && !EMAIL_REGEX.is_match(&self.person) {
            return Err("Invalid email format".to_string());
        }
        if self.password.is_empty() {
            return Err("Password is required".to_string());
        }

        Ok(())
    }
}
