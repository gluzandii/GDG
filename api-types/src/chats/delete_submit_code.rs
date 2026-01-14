use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeleteSubmitCodeRequest {
    /// The chat code to be deleted.
    pub code: u16,
}
