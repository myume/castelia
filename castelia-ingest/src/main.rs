use std::env;

use async_trait::async_trait;
use castelia_auth::routes::stream_key::{VerifyStreamKeyRequest, VerifyStreamKeyResponse};
use castelia_rtmp::{
    broadcast::authenticator::{AuthenticateError, Authenticator},
    rtmp::RTMPServer,
};
use dotenvy::dotenv;
use reqwest::StatusCode;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let auth_url = env::var("AUTH_URL")?;
    let authenticator = ApiAuth::new(&auth_url);

    tracing_subscriber::fmt::init();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1935").await?;
    info!("Listening on {}", listener.local_addr()?);

    RTMPServer::new(listener, Box::new(authenticator))
        .run()
        .await?;

    Ok(())
}

struct ApiAuth {
    client: reqwest::Client,
    auth_url: String,
}

impl ApiAuth {
    pub fn new(auth_url: &str) -> Self {
        Self {
            auth_url: auth_url.to_owned(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Authenticator for ApiAuth {
    async fn authenticate(&self, stream_key: &str) -> Result<String, AuthenticateError> {
        let res = self
            .client
            .post(format!("{}/streamkey", self.auth_url))
            .json(&VerifyStreamKeyRequest {
                stream_key: stream_key.to_owned(),
            })
            .send()
            .await
            .map_err(|e| {
                error!("Error sending stream key: {e}");
                AuthenticateError::SendError
            })?;

        if res.status() == StatusCode::UNAUTHORIZED {
            return Err(AuthenticateError::InvalidKey);
        }

        let res: VerifyStreamKeyResponse = res
            .json()
            .await
            .map_err(|_| AuthenticateError::UnexpectedResponse)?;
        Ok(res.username)
    }
}
