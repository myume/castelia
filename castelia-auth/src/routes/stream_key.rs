use aes_gcm::{
    AeadCore, Aes256Gcm, KeyInit,
    aead::{Aead, OsRng, rand_core::RngCore},
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use base64::Engine;
use hmac::Mac;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{AppState, routes::Claims};

pub fn encrypt_stream_key(
    stream_key: &str,
    encryption_key: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), String> {
    let cipher = Aes256Gcm::new(encryption_key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted_stream_key = cipher
        .encrypt(&nonce, stream_key.as_bytes().as_ref())
        .map_err(|_| "Failed to encrypt stream key")?;

    Ok((encrypted_stream_key, nonce.to_vec()))
}

pub fn hash_stream_key(stream_key: &str, encryption_key: &[u8]) -> Result<Vec<u8>, String> {
    let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::Mac>::new_from_slice(encryption_key)
        .map_err(|_| "Failed to initialize HMAC hash")?;

    mac.update(stream_key.as_bytes());
    Ok(mac.finalize().into_bytes().to_vec())
}

pub fn generate_stream_key() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|_| "Could not generate stream key")?;

    let secret = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    let stream_key = format!("cast_{}", secret);

    Ok(stream_key)
}
fn decrypt_stream_key(
    encrypted_stream_key: &[u8],
    nonce: &[u8],
    encryption_key: &[u8],
) -> Result<String, String> {
    let cipher = Aes256Gcm::new(encryption_key.into());
    let stream_key = cipher
        .decrypt(nonce.into(), encrypted_stream_key.as_ref())
        .map_err(|_| "Failed to decrypt stream key")?;

    Ok(String::from_utf8(stream_key).map_err(|_| "Invalid UTF8")?)
}

pub enum StreamKeyError {
    NotFound,
    DecryptError(String),
}

impl IntoResponse for StreamKeyError {
    fn into_response(self) -> axum::response::Response {
        match self {
            StreamKeyError::NotFound => {
                error!("Stream key not found for user");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            StreamKeyError::DecryptError(err) => {
                error!("Failed to decrypt stream key: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub async fn get_streamkey(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<String, StreamKeyError> {
    struct StreamKey {
        stream_key: Vec<u8>,
        nonce: Vec<u8>,
    }
    let row = sqlx::query_as!(
        StreamKey,
        "SELECT stream_key, nonce FROM users WHERE id = $1",
        claims.sub
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StreamKeyError::NotFound)?;

    let stream_key = decrypt_stream_key(&row.stream_key, &row.nonce, &state.encryption_key)
        .map_err(StreamKeyError::DecryptError)?;

    Ok(stream_key)
}

#[derive(Debug, Deserialize)]
pub struct VerifyStreamKeyRequest {
    stream_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyStreamKeyResponse {
    username: String,
}

pub async fn verify_streamkey(
    State(state): State<AppState>,
    Json(VerifyStreamKeyRequest { stream_key }): Json<VerifyStreamKeyRequest>,
) -> Result<Json<VerifyStreamKeyResponse>, StatusCode> {
    let hashed_stream_key = hash_stream_key(&stream_key, &state.encryption_key).map_err(|_| {
        error!("Unable to hash stream key");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    struct Row {
        username: String,
        stream_key: Vec<u8>,
        nonce: Vec<u8>,
    }
    // should we handle collisions? something to think about.
    let row = sqlx::query_as!(
        Row,
        "SELECT username, stream_key, nonce FROM users WHERE stream_key_hash = $1",
        hashed_stream_key
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let expected_stream_key =
        decrypt_stream_key(&row.stream_key, &row.nonce, &state.encryption_key)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if expected_stream_key != stream_key {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(Json(VerifyStreamKeyResponse {
        username: row.username,
    }))
}
