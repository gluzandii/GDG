//! Chat route handlers.
//!
//! This module contains all chat-related endpoints including creation,
//! deletion, and real-time WebSocket communication.

/// Create new chat endpoint handler.
pub mod new_code;

/// Delete chat endpoint handler.
pub mod delete_code;

/// Submit chat code endpoint handler.
pub mod submit_code;

/// WebSocket real-time chat handler.
pub mod ws;
