use serde::Serialize;
use time::OffsetDateTime;

#[derive(Serialize)]
pub struct UsersMeResponse {
    pub email: String,
    pub username: String,
    pub bio: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
