use aes_gcm::{
    AeadCore, Aes256Gcm, KeyInit,
    aead::{Aead, OsRng},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use base64::Engine;
use rand::TryRngCore;
use tracing::error;

use crate::{AppState, routes::Claims};

pub fn generate_stream_key(encryption_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|_| "Could not generate stream key")?;
    let secret = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    let stream_key = format!("cast_{}", secret);
    let cipher = Aes256Gcm::new(encryption_key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted_stream_key = cipher
        .encrypt(&nonce, stream_key.as_bytes().as_ref())
        .map_err(|_| "Failed to encrypt stream key")?;
    Ok((encrypted_stream_key, nonce.to_vec()))
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
    claims: Claims,
    State(state): State<AppState>,
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

pub async fn verify_streamkey() {}
