//! Create new chat request and response types.

use serde::Serialize;

/// Response payload for successful chat creation.
///
/// Contains the chat code and a success message for the newly created chat.
#[derive(Serialize)]
pub struct CreateChatResponse {
    /// Success message for chat creation.
    pub message: String,
    /// The unique code for the created chat.
    pub code: u16,
}
