use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

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
    (status, Json(json!({ "error": message.as_ref() }))).into_response()
}
