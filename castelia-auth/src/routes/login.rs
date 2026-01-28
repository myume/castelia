use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use chrono::{TimeDelta, Utc};
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{AppState, routes::Claims};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("Invalid username or password.")]
    BadUsernameOrPassword,

    #[error(transparent)]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error(transparent)]
    JWTEncoding(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    InvalidUserId(#[from] uuid::Error),
}

impl IntoResponse for LoginError {
    fn into_response(self) -> axum::response::Response {
        match self {
            LoginError::BadUsernameOrPassword => (
                StatusCode::BAD_REQUEST,
                "Invalid username or password.".to_string(),
            )
                .into_response(),
            LoginError::PasswordHashError(error) => {
                error!("Could not read hashed password: {error}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            LoginError::JWTEncoding(error) => {
                error!("Failed to encode JWT: {error}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            LoginError::InvalidUserId(error) => {
                error!("Could not parse user id as uuid: {error}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

struct User {
    id: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    access_token: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(login): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, LoginError> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, password FROM users WHERE username = $1",
        login.username
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| LoginError::BadUsernameOrPassword)?;

    Argon2::default()
        .verify_password(
            login.password.as_bytes(),
            &PasswordHash::new(&user.password)?,
        )
        .map_err(|_| LoginError::BadUsernameOrPassword)?;

    let exp = Utc::now()
        .checked_add_signed(TimeDelta::hours(1))
        .unwrap_or(Utc::now());
    let claims = Claims {
        sub: uuid::Uuid::parse_str(&user.id)?,
        exp: exp.timestamp() as usize,
        username: login.username,
    };

    let access_token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.encryption_key.as_ref()),
    )?;

    Ok(Json(LoginResponse { access_token }))
}
