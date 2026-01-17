//! WebSocket connection types.

use serde::Deserialize;
use uuid::Uuid;

/// Query parameters for WebSocket connections.
#[derive(Deserialize)]
pub struct ApiChatsWsQuery {
    /// The chat code to connect to
    #[serde(rename = "chatId")]
    pub chat_id: Option<Uuid>,
}
