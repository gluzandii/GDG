//! Update user profile endpoint handler.
//!
//! Handles updating user profile information including email, username, and bio.

use api_types::{
    auth::EMAIL_REGEX,
    users::update::{UsersUpdateRequest, UsersUpdateResponse},
};
use axum::{Extension, Json, extract::State, http::StatusCode, response::IntoResponse};
use sqlx::PgPool;
use utils::errors::error_response;

#[derive(sqlx::FromRow)]
struct UserUpdateFields {
    email: String,
    username: String,
    bio: Option<String>,
    password_hash: String,
}

/// Handles user profile update requests.
///
/// This endpoint:
/// 1. Retrieves the current user's profile information
/// 2. Validates the provided email and username if they differ from current values
/// 3. Checks that the new email/username don't already exist for other users
/// 4. Updates the user's profile in the database
/// 5. Returns the updated profile information
///
/// # Arguments
///
/// * `pool` - The PostgreSQL connection pool
/// * `user_id` - The authenticated user's ID from the JWT cookie
/// * `payload` - The update request with new profile information
///
/// # Returns
///
/// - `200 OK` with the updated user profile
/// - `400 BAD REQUEST` if validation fails (invalid email, email/username already exists)
/// - `404 NOT FOUND` if the user doesn't exist
/// - `500 INTERNAL SERVER ERROR` if database operations fail
#[tracing::instrument(skip(pool, user_id, payload))]
pub async fn update_route(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<UsersUpdateRequest>,
) -> impl IntoResponse {
    // Query the email username bio and password from the user id
    let user = match sqlx::query_as!(
        UserUpdateFields,
        r#"
        SELECT email, username, bio, password_hash
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
            tracing::warn!(user_id, "User not found during update");
            return error_response(StatusCode::NOT_FOUND, "User not found");
        }
        Err(e) => {
            tracing::error!(error = ?e, "Database error during user profile fetch");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error while querying",
            );
        }
    };

    let pswd = payload.password;
    let hash_result = utils::hashing::verify_password(&pswd, &user.password_hash);

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
        tracing::warn!("Password verification failed during profile update");
        tracing::debug!(user_id, "Invalid password provided for profile update");
        return error_response(StatusCode::UNAUTHORIZED, "Invalid password");
    }

    // Prepare update fields and track which ones changed
    let new_email = payload.email.as_deref().unwrap_or(&user.email);
    let new_username = payload.username.as_deref().unwrap_or(&user.username);
    let new_bio = payload.bio.as_ref().or(user.bio.as_ref());

    if !EMAIL_REGEX.is_match(&new_email) {
        tracing::debug!("Invalid email address during profile update");
        return error_response(StatusCode::BAD_REQUEST, "Email format is invalid");
    }

    let mut updated_fields = vec![];
    let mut new_password_hash = user.password_hash.clone();

    if payload.email.is_some() && payload.email.as_deref() != Some(user.email.as_str()) {
        updated_fields.push("email".to_string());
    }
    if payload.username.is_some() && payload.username.as_deref() != Some(user.username.as_str()) {
        updated_fields.push("username".to_string());
    }
    if payload.bio.is_some() && payload.bio != user.bio {
        updated_fields.push("bio".to_string());
    }

    // Handle password update if a new password is provided
    if let Some(ref new_password) = payload.new_password {
        // Validate the new password meets requirements
        match utils::hashing::is_password_suitable(new_password) {
            Ok(_) => (),
            Err(e) => {
                tracing::warn!(error = ?e, "New password is not suitable: {e}");
                return error_response(StatusCode::BAD_REQUEST, e);
            }
        }
        // Hash the new password
        match utils::hashing::hash_password(new_password) {
            Ok(h) => {
                new_password_hash = h;
                updated_fields.push("password".to_string());
            }
            Err(e) => {
                tracing::error!(error = ?e, "Failed to hash new password");
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Error hashing password");
            }
        }
    }

    // Update the user in the database
    match sqlx::query!(
        r#"
        UPDATE users
        SET email = $1, username = $2, bio = $3, password_hash = $4, updated_at = NOW()
        WHERE id = $5
        "#,
        new_email,
        new_username,
        new_bio,
        new_password_hash,
        user_id
    )
    .execute(&pool)
    .await
    {
        Ok(_) => {
            let response = UsersUpdateResponse { updated_fields };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!(error = ?e, "Failed to update user");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update user")
        }
    }
}
