//! Chat-related API types.
//!
//! This module contains all chat-related request and response types
//! for creating, deleting, and communicating in chat conversations.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Create new chat endpoint types.
pub mod new_code;

/// Delete and submit code endpoint types.
pub mod delete_submit_code;

/// WebSocket chat communication types.
pub mod ws;

/// Query parameters for retrieving chats.
///
/// Supports cursor-based pagination using `cursor` and `limit`.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetChatsQuery {
    /// The conversation ID to retrieve messages from.
    pub conversation_id: Uuid,
    /// Timestamp cursor (RFC3339). Returns messages sent before this cursor when provided.
    pub cursor: Option<String>,
    /// Maximum number of messages to return. Defaults to 50 and capped at 100.
    pub limit: Option<i64>,
}

/// Response payload for successful chats retrieval.
///
/// Contains a list of chat codes and their metadata.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChatsResponse {
    /// List of chats belonging to the user.
    pub chats: Vec<ChatItem>,
    /// Cursor for fetching the next page (older messages). None when there are no more.
    pub next_cursor: Option<String>,
    /// Indicates whether another page exists.
    pub has_more: bool,
}

/// Represents a single chat message item in the response.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatItem {
    /// Unique identifier for the message.
    pub id: Uuid,
    /// The message content.
    pub content: String,
    /// The user who sent the message.
    pub user_sent: String,
    /// Timestamp when the message was sent.
    pub sent_at: String,
}

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
