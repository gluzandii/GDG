use argon2::Argon2;
use password_hash::rand_core::OsRng;
use password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

pub fn hash_password<S: AsRef<str>>(password: S) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_ref().as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

pub fn verify_password<S: AsRef<str>>(
    password: S,
    stored_hash: S,
) -> Result<bool, password_hash::Error> {
    let parsed_hash = PasswordHash::new(stored_hash.as_ref())?;
    Ok(Argon2::default()
        .verify_password(password.as_ref().as_bytes(), &parsed_hash)
        .is_ok())
}
