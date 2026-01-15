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
/// All parameters are optional. When `all` is true, returns all messages.
/// Otherwise, returns messages filtered by the `from` and `to` timestamps.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetChatsQuery {
    /// The conversation ID to retrieve messages from.
    pub conversation_id: Uuid,
    /// If true, retrieves all messages, overriding from and to filters.
    pub all: Option<bool>,
    /// Start timestamp for message filtering (inclusive, RFC3339 format). If omitted, retrieves from the beginning.
    pub from: Option<String>,
    /// End timestamp for message filtering (inclusive, RFC3339 format). If omitted, retrieves till the end.
    pub to: Option<String>,
}

/// Response payload for successful chats retrieval.
///
/// Contains a list of chat codes and their metadata.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetChatsResponse {
    /// List of chats belonging to the user.
    pub chats: Vec<ChatItem>,
}

/// Represents a single chat message item in the response.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatItem {
    /// The message content.
    pub content: String,
    /// The user who sent the message.
    pub user_sent: String,
    /// Timestamp when the message was sent.
    pub sent_at: String,
}
