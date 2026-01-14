//! Create new chat request and response types.

use serde::Serialize;

/// Response payload for successful chat creation.
///
/// Contains the chat code for the newly created chat.
#[derive(Serialize)]
pub struct CreateChatResponse {
    /// The unique code for the created chat.
    pub code: u32,
}
