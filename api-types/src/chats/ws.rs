//! WebSocket connection types.

use serde::Deserialize;

/// Query parameters for WebSocket connections.
#[derive(Deserialize)]
pub struct ChatQuery {
    /// The chat code to connect to
    #[serde(rename = "chatCode")]
    pub chat_code: Option<u16>,
}
