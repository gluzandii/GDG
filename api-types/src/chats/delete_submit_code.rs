use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DeleteSubmitCodeRequest {
    /// The chat code to be deleted or submitted.
    pub code: u16,
}

/// Response payload for successful code operations (deletion or submission).
#[derive(Serialize)]
pub struct DeleteSubmitCodeResponse {
    /// Success message for the operation.
    pub message: String,
}
