use api_types::auth::register::{RegisterRequest, RegisterResponse};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use sqlx::PgPool;
use utils::hashing;

pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    if let Err(e) = req.validate() {
        let resp = RegisterResponse {
            ok: false,
            message: e,
            id: None,
        };

        return (StatusCode::BAD_REQUEST, Json(resp));
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
            let resp = RegisterResponse {
                ok: false,
                message: format!("A database error occurred on our end: {}", e),
                id: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(resp));
        }
    };

    if existing.username_exists && existing.email_exists {
        let resp = RegisterResponse {
            ok: false,
            message: "This user already exists.".to_string(),
            id: None,
        };
        return (StatusCode::CONFLICT, Json(resp));
    }
    if existing.username_exists {
        let resp = RegisterResponse {
            ok: false,
            message: "Username already exists".to_string(),
            id: None,
        };
        return (StatusCode::CONFLICT, Json(resp));
    }
    if existing.email_exists {
        let resp = RegisterResponse {
            ok: false,
            message: "Email already exists".to_string(),
            id: None,
        };
        return (StatusCode::CONFLICT, Json(resp));
    }

    let hashed = match hashing::hash_password(password) {
        Ok(h) => h,
        Err(e) => {
            let resp = RegisterResponse {
                ok: false,
                message: format!("An error occurred on our end: {}", e),
                id: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(resp));
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
            let resp = RegisterResponse {
                ok: false,
                message: format!("A database error occurred on our end: {}", e),
                id: None,
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(resp));
        }
    };

    let resp = RegisterResponse {
        ok: true,
        message: "".to_string(),
        id: Some(user.id),
    };
    (StatusCode::OK, Json(resp))
}
