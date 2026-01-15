use api_types::chats::ws::ChatQuery;
use axum::http::StatusCode;
use axum::{
    extract::{Query, State, ws::WebSocketUpgrade},
    response::IntoResponse,
};
use sqlx::PgPool;
use utils::errors::error_response;

pub async fn ws_handler(
    Query(params): Query<ChatQuery>,
    ws: WebSocketUpgrade,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let chat_code = match params.chat_code {
        Some(code) => {
            println!("Chat code: {}", code);
            code
        }
        None => return error_response(StatusCode::BAD_REQUEST, "Chat code not provided"),
    };

    ws.on_upgrade(async move |socket| println!("hi"))
}
