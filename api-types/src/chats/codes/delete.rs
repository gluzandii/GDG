use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ApiChatsCodesDeleteRequest {
    /// The chat code to be deleted or submitted.
    pub code: u16,
}

/// Response payload for successful code operations (deletion or submission).
#[derive(Serialize)]
pub struct ApiChatsCodeDeleteResponse {
    /// Success message for the operation.
    pub message: String,

    /// The ID of the created conversation (only for submission).
    pub conversation_id: Option<Uuid>,
}
