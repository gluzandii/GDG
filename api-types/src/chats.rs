//! Chat-related API types.
//!
//! This module contains all chat-related request and response types
//! for creating, deleting, and communicating in chat conversations.

/// Create new chat endpoint types.
pub mod new_code;

/// Delete and submit code endpoint types.
pub mod delete_submit_code;

/// WebSocket chat communication types.
pub mod ws;

/// Get messages endpoint types.
pub mod get;

/// Delete message endpoint types.
pub mod delete;

/// Patch message endpoint types.
pub mod patch;

pub use delete::{DeleteMessageRequest, DeleteMessageResponse};
pub use get::{ChatItem, GetChatsQuery, GetChatsResponse};
pub use patch::{UpdateMessageRequest, UpdateMessageResponse};
