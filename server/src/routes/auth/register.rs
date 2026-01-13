use api_types::auth::register::{RegisterRequest, RegisterResponse};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use sqlx::PgPool;
use utils::hashing;

#[inline(always)]
fn error_response(status: StatusCode, message: String) -> axum::response::Response {
    let resp = RegisterResponse {
        ok: false,
        message,
        id: None,
    };
    (status, Json(resp)).into_response()
}

pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        return error_response(
            StatusCode::UNAUTHORIZED,
            format!("Your request was invalid: {}", e),
        );
    }

    let RegisterRequest {
        username,
        email,
        password,
    } = req;

    // Check if username or email already exists
    let existing = match sqlx::query!(
        r#"
        SELECT
            EXISTS(SELECT 1 FROM users WHERE username = $1) as "username_exists!",
            EXISTS(SELECT 1 FROM users WHERE email = $2) as "email_exists!"
        "#,
        username,
        email
    )
    .fetch_one(&pool)
    .await
    {
        Ok(record) => record,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("A database error occurred on our end: {}", e),
            );
        }
    };

    if existing.username_exists && existing.email_exists {
        return error_response(
            StatusCode::CONFLICT,
            "This user already exists.".to_string(),
        );
    }
    if existing.username_exists {
        return error_response(StatusCode::CONFLICT, "Username already exists".to_string());
    }
    if existing.email_exists {
        return error_response(StatusCode::CONFLICT, "Email already exists".to_string());
    }

    let hashed = match hashing::hash_password(password) {
        Ok(h) => h,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("An error occurred on our end: {}", e),
            );
        }
    };

    let user = match sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        username,
        email,
        hashed
    )
    .fetch_one(&pool)
    .await
    {
        Ok(record) => record,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("A database error occurred on our end: {}", e),
            );
        }
    };

    let resp = RegisterResponse {
        ok: true,
        message: "".to_string(),
        id: Some(user.id),
    };
    (StatusCode::CREATED, Json(resp)).into_response()
}
