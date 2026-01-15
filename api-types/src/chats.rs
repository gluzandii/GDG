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
