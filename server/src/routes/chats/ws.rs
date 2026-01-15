use axum::{
    extract::{Path, State, ws::WebSocketUpgrade},
    response::IntoResponse,
};
use sqlx::PgPool;

pub async fn ws_handler(
    Path(chat_code): Path<String>,
    ws: WebSocketUpgrade,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    ws.on_upgrade(async move |socket| println!("hi"))
}
