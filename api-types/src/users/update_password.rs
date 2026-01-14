use serde::{Deserialize, Serialize};

/// Request structure for updating user password.
/// This structure captures the old password for verification
/// and the new password that will replace it.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePasswordRequest {
    /// The user's current password for verification
    pub old_password: String,
    /// The new password to set
    pub new_password: String,
}

/// Response structure for password update operations.
#[derive(Serialize)]
pub struct UpdatePasswordResponse {
    /// A message providing details about the operation (success or failure reason).
    pub message: String,
}
