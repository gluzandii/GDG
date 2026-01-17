use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request payload for deleting a chat message.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteMessageRequest {
    /// Conversation that the message belongs to.
    pub conversation_id: Uuid,
    /// The message to delete.
    pub message_id: Uuid,
}

/// Response payload for deleting a chat message.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteMessageResponse {
    /// Confirmation message.
    pub message: String,
}
