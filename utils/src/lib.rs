//! # Utils
//!
//! This crate provides utility functions for the GDG realtime chat application.
//! It includes password hashing and JWT token management.

/// Password hashing and verification utilities using Argon2.
pub mod hashing;

/// JWT token generation, verification, and cookie building utilities.
pub mod jwt;

/// Error types and handling utilities.
pub mod errors;
