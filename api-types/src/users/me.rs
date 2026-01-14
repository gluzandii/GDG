//! User profile response types.

use serde::Serialize;
use time::OffsetDateTime;

/// Response payload for the authenticated user's profile.
///
/// Contains the current user's profile information including
/// email, username, bio, and timestamps.
#[derive(Serialize)]
pub struct MeResponse {
    /// The user's email address.
    pub email: String,
    /// The user's username.
    pub username: String,
    /// The user's optional biography/description.
    pub bio: Option<String>,
    /// Timestamp when the user account was created.
    pub created_at: OffsetDateTime,
    /// Timestamp when the user account was last updated.
    pub updated_at: OffsetDateTime,
}
