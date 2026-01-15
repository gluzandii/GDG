//! Middleware utilities for the application.
//!
//! This module provides middleware for request processing, including
//! authentication verification and JWT token validation.

/// JWT authentication middleware for protecting routes.
pub mod auth;

pub use auth::auth_middleware;
