use std::str;

use serde::Deserialize;

/// Request structure for updating user information.
/// This structure is used to capture the fields that a user
/// wants to update in their profile.
/// All fields are optional, allowing partial updates.
#[derive(Deserialize)]
pub struct UsersUpdateRequest {
    /// The user's email address.
    pub email: Option<String>,
    /// The user's username.
    pub username: Option<String>,
    /// The user's optional biography/description.
    pub bio: Option<String>,
    /// Password for verification before sensitive operations.
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct UsersUpdateResponse {
    /// List of fields that were updated.
    #[serde(rename = "updatedFields")]
    pub updated_fields: Vec<String>,
}
