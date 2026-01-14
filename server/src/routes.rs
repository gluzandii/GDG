//! Route modules for the server.
//!
//! This module organizes all route handlers by feature.

/// Authentication routes (registration, login, etc.).
pub mod auth;

/// Chat management routes (create, list, etc.).
pub mod chats;

/// User management routes (profile, settings, etc.).
pub mod users;
