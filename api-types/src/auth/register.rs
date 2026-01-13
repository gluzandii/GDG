use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct RegisterResponse {
    pub ok: bool,
    pub message: String,
    pub id: Option<i64>,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), String> {
        let email_re = Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$");

        let email_re = match email_re {
            Ok(re) => re,
            Err(e) => {
                tracing::debug!(error = ?e, "Regex compilation failed");
                return Err("Failed to compile email regex".into());
            }
        };

        if !email_re.is_match(&self.email) {
            tracing::debug!("Invalid email address");
            return Err("Email format is invalid".into());
        }

        // NOTE: Rust's `regex` crate does NOT support look-around (no look-ahead / look-behind).
        // So we validate password rules with simple character checks.
        let password = self.password.as_str();

        if password.len() < 6 {
            tracing::debug!("Password is too short");
            return Err("Password must be at least 6 characters".into());
        }

        let mut chars = password.chars();
        let has_upper = chars.any(|c| c.is_ascii_uppercase());
        let has_lower = chars.any(|c| c.is_ascii_lowercase());
        let has_digit = chars.any(|c| c.is_ascii_digit());

        if !(has_upper && has_lower && has_digit) {
            tracing::debug!("Password does not meet complexity requirements");
            return Err("Password must contain at least one uppercase letter, one lowercase letter, and one digit".into());
        }

        Ok(())
    }
}
