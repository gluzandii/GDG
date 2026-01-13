//! Password hashing and verification using Argon2.
//!
//! This module provides secure password hashing and verification functionality
//! using the Argon2 algorithm with random salts.

use argon2::Argon2;
use password_hash::rand_core::OsRng;
use password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use tracing::trace;

/// Hashes a password using Argon2 with a random salt.
///
/// # Arguments
///
/// * `password` - The plaintext password to hash
///
/// # Returns
///
/// - `Ok(String)` containing the hashed password in PHC format
/// - `Err(password_hash::Error)` if hashing fails
///
/// # Example
///
/// ```ignore
/// let hashed = hash_password("my_password")?;
/// ```
pub fn hash_password<S: AsRef<str>>(password: S) -> Result<String, password_hash::Error> {
    trace!("Hashing password");
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_ref().as_bytes(), &salt)?
        .to_string();
    trace!("Password hashed successfully");
    Ok(hash)
}

/// Verifies a password against a stored hash.
///
/// # Arguments
///
/// * `password` - The plaintext password to verify
/// * `stored_hash` - The stored password hash in PHC format
///
/// # Returns
///
/// - `Ok(true)` if the password matches the hash
/// - `Ok(false)` if the password does not match
/// - `Err(password_hash::Error)` if verification fails due to invalid hash format
///
/// # Example
///
/// ```ignore
/// let is_valid = verify_password("my_password", &stored_hash)?;
/// ```
pub fn verify_password<S: AsRef<str>>(
    password: S,
    stored_hash: S,
) -> Result<bool, password_hash::Error> {
    trace!("Verifying password");
    let parsed_hash = PasswordHash::new(stored_hash.as_ref())?;
    let result = Argon2::default()
        .verify_password(password.as_ref().as_bytes(), &parsed_hash)
        .is_ok();
    trace!(success = result, "Password verification completed");
    Ok(result)
}
