use crate::routes::auth::register::serde::{RegisterRequest, RegisterResponse};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

pub async fn register(Json(req): Json<RegisterRequest>) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        let resp = RegisterResponse {
            ok: false,
            message: e,
        };

        return (StatusCode::BAD_REQUEST, Json(resp));
    }

    let resp = RegisterResponse {
        ok: true,
        message: "todo".to_string(),
    };
    (StatusCode::OK, Json(resp))
}
