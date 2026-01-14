use axum::{http::StatusCode, response::IntoResponse};

/// Creates an error response with the specified status code and message.
///
/// # Arguments
///
/// * `status` - The HTTP status code
/// * `message` - The error message to return
///
/// # Returns
///
/// An Axum response with the error details in JSON format.
#[inline(always)]
pub fn error_response<S: AsRef<str>>(status: StatusCode, message: S) -> axum::response::Response {
    let json_body = format!(r#"{{"error":"{}"}}"#, message.as_ref());
    (status, json_body).into_response()
}
