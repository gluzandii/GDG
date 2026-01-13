use jsonwebtoken::{
    decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

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
