//! Chat route handlers.
//!
//! This module contains all chat-related endpoints including creation,
//! deletion, and real-time WebSocket communication.

/// Submit chat code endpoint handler.
pub mod post;

pub mod messages;

/// WebSocket real-time chat handler.
pub mod ws;

pub mod codes;
