//! Delete chat code request and response types.

use serde::Serialize;

/// Response payload for successful chat code deletion.
///
/// Contains a success message.
#[derive(Serialize)]
pub struct DeleteCodeResponse {
    /// Success message for code deletion.
    pub message: String,
}
