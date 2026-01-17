use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request payload for updating a chat message.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiChatsMessagesPatchRequest {
    /// Conversation that the message belongs to.
    pub conversation_id: Uuid,
    /// The message to update.
    pub message_id: Uuid,
    /// The new message content.
    pub content: String,
}

/// Response payload for updating a chat message.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiChatsMessagesPatchResponse {
    /// Confirmation message.
    pub message: String,
    /// Timestamp when the message was last edited.
    pub edited_at: String,
}
