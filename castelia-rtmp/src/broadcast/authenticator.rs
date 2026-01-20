use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthenticateError {
    #[error("Invalid Stream Key")]
    InvalidKey,
}

#[async_trait]
pub trait Authenticator: Send + Sync {
    async fn authenticate(&self, stream_key: &str) -> Result<String, AuthenticateError>;
}
