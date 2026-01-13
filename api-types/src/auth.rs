//! Authentication API types.
//!
//! This module contains all authentication-related request and response types.

use once_cell::sync::Lazy;
use regex::Regex;

pub mod login;
/// User registration types and validation.
pub mod register;

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$")
        .expect("Regex compilation failed")
});
