use argon2::Argon2;
use password_hash::rand_core::OsRng;
use password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use tracing::trace;

pub fn hash_password<S: AsRef<str>>(password: S) -> Result<String, password_hash::Error> {
    trace!("Hashing password");
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_ref().as_bytes(), &salt)?
        .to_string();
    trace!("Password hashed successfully");
    Ok(hash)
}

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
