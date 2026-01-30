use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use chrono::{TimeDelta, Utc};
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    AppState,
    routes::{Claims, RefreshClaims, TokenType},
};

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

    #[error("Could not set expiration for JWT")]
    InvalidExpiration,
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
            LoginError::InvalidExpiration => {
                error!("{self}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    access_token: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(login): Json<LoginRequest>,
) -> Result<impl IntoResponse, LoginError> {
    let user = sqlx::query!(
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

    // generate access token
    let exp = Utc::now()
        .checked_add_signed(TimeDelta::minutes(30))
        .ok_or(LoginError::InvalidExpiration)?;
    let claims = Claims {
        sub: user.id,
        exp: exp.timestamp(),
        username: login.username,
        token_type: TokenType::Access,
    };
    let access_token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.encryption_key.as_ref()),
    )?;

    // generate refresh token
    let exp = Utc::now()
        .checked_add_signed(TimeDelta::days(7))
        .ok_or(LoginError::InvalidExpiration)?;
    let refresh_claims = RefreshClaims {
        exp: exp.timestamp(),
        sub: user.id,
        token_type: TokenType::Refresh,
    };
    let refresh_token = jsonwebtoken::encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(state.encryption_key.as_ref()),
    )?;

    let cookie = Cookie::build(("refresh_token", refresh_token))
        .secure(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .http_only(true)
        .max_age(time::Duration::days(7))
        .path("/");

    Ok((jar.add(cookie), Json(LoginResponse { access_token })))
}
