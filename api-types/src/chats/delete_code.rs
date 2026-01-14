use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeleteChatCodeRequest {
    /// The chat code to be deleted.
    pub code: u16,
}
