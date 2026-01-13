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
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), String> {
        let email_re = Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$")
            .expect("Regex failed to compile");

        if !email_re.is_match(&self.email) {
            return Err("Email format is invalid".into());
        }

        let password = &self.password;

        let password_re = Regex::new(r"^(?=.*[A-Z])(?=.*[a-z])(?=.*[0-9]).{6,}$").unwrap();
        if !password_re.is_match(password) {
            return Err(
                "password must be at least 6 characters and contain upper, lower, and digit".into(),
            );
        }

        Ok(())
    }
}
