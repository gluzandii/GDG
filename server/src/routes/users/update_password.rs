use api_types::users::update_password::UpdatePasswordRequest;
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use utils::errors::error_response;

#[derive(sqlx::FromRow)]
struct UserPasswordFields {
    password_hash: String,
}

#[tracing::instrument(skip(pool, user_id, payload))]
pub async fn update_password_route(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> impl IntoResponse {
    // Query the password hash for the user
    let user = match sqlx::query_as!(
        UserPasswordFields,
        r#"
        SELECT password_hash
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!(user_id, "User not found during password update");
            return error_response(StatusCode::NOT_FOUND, "User not found");
        }
        Err(e) => {
            tracing::error!(error = ?e, "Database error during password fetch");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error while querying",
            );
        }
    };

    // Verify the old password
    let old_pswd = payload.old_password;
    let hash_result = utils::hashing::verify_password(&old_pswd, &user.password_hash);

    let verified = match hash_result {
        Ok(valid) => valid,
        Err(e) => {
            tracing::error!(error = ?e, "An error occurred while verifying password");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error during password verification",
            );
        }
    };

    if !verified {
        tracing::warn!("Password verification failed during password update");
        tracing::debug!(user_id, "Invalid old password provided");
        return error_response(StatusCode::UNAUTHORIZED, "Invalid old password");
    }

    // Hash the new password
    let new_pswd = payload.new_password;
    let hashed = match utils::hashing::hash_password(new_pswd) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!(error = ?e, "Failed to hash new password");
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Error hashing password");
        }
    };

    // Update the password in the database
    match sqlx::query!(
        r#"
        UPDATE users
        SET password_hash = $1, updated_at = NOW()
        WHERE id = $2
        "#,
        hashed,
        user_id
    )
    .execute(&pool)
    .await
    {
        Ok(_) => {
            tracing::debug!("Password updated successfully for user");
            let json_body = r#"{"message":"Password updated successfully"}"#;
            (StatusCode::OK, json_body).into_response()
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to update password");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to update password",
            )
        }
    }
}
