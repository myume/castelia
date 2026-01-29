use aes_gcm::aead::OsRng;
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{PasswordHashString, SaltString},
};
use axum::{
    Json,
    extract::{FromRequest, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tracing::error;
use validator::Validate;

use crate::{
    AppState,
    routes::stream_key::{encrypt_stream_key, generate_stream_key, hash_stream_key},
};

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 3, max = 32))]
    username: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 6))]
    password: String,
}

pub struct ValidatedUser(CreateUser);
impl<S> FromRequest<S> for ValidatedUser
where
    S: Send + Sync,
{
    type Rejection = CreateUserError;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(user) = Json::<CreateUser>::from_request(req, state).await?;
        user.validate()?;
        Ok(ValidatedUser(user))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateUserError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error("Failed to hash password")]
    PasswordHashFailure,

    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    InsertionError(#[from] sqlx::Error),

    #[error("Failed to generate stream key")]
    StreamKeyGenerationFailure(String),
}

impl IntoResponse for CreateUserError {
    fn into_response(self) -> Response {
        match self {
            CreateUserError::ValidationError(validation_errors) => {
                (StatusCode::BAD_REQUEST, validation_errors.to_string()).into_response()
            }
            CreateUserError::PasswordHashFailure => {
                error!("Failed to hash password");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            CreateUserError::JsonRejection(json_rejection) => json_rejection.into_response(),
            CreateUserError::InsertionError(error) => match error.as_database_error() {
                Some(error) if error.is_unique_violation() => {
                    error!("Creating duplicate user: {error}");
                    (StatusCode::BAD_REQUEST, "User already exists").into_response()
                }
                _ => {
                    error!("Failed to insert user into database: {error}");
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            },
            CreateUserError::StreamKeyGenerationFailure(error) => {
                error!("Failed to generate stream key: {error}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub async fn signup(
    State(state): State<AppState>,
    ValidatedUser(user): ValidatedUser,
) -> Result<StatusCode, CreateUserError> {
    let password_hash =
        tokio::task::spawn_blocking(move || -> Result<PasswordHashString, CreateUserError> {
            let salt = SaltString::generate(&mut OsRng);
            let password_hash = Argon2::default()
                .hash_password(user.password.as_bytes(), &salt)
                .map_err(|_| CreateUserError::PasswordHashFailure)?;
            Ok(password_hash.serialize())
        })
        .await
        .map_err(|_| CreateUserError::PasswordHashFailure)??;

    let stream_key = generate_stream_key().map_err(CreateUserError::StreamKeyGenerationFailure)?;
    let (encrypted_stream_key, nonce) = encrypt_stream_key(&stream_key, &state.encryption_key)
        .map_err(CreateUserError::StreamKeyGenerationFailure)?;
    let stream_key_hash = hash_stream_key(&stream_key, &state.encryption_key)
        .map_err(CreateUserError::StreamKeyGenerationFailure)?;

    sqlx::query!(
        r#"INSERT INTO users (username, email, password, stream_key, nonce, stream_key_hash) VALUES ($1, $2, $3, $4, $5, $6)"#,
        user.username,
        user.email,
        password_hash.as_str(),
        encrypted_stream_key,
        nonce,
        stream_key_hash
    )
    .execute(&state.db)
    .await?;

    Ok(StatusCode::CREATED)
}
