use serde::Deserialize;

#[derive(Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}
