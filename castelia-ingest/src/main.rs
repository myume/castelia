use std::env;

use anyhow::Context;
use async_trait::async_trait;
use castelia_auth::routes::stream_key::{VerifyStreamKeyRequest, VerifyStreamKeyResponse};
use castelia_events::stream_events::{STREAM_EVENT_KEY, StreamEvent, StreamEventEmitter};
use castelia_rtmp::{
    broadcast::authenticator::{AuthenticateError, Authenticator},
    rtmp::RTMPServer,
};
use dotenvy::dotenv;
use redis::Commands;
use reqwest::StatusCode;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let auth_url = env::var("AUTH_URL")?;

    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:1935";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on {}", listener.local_addr()?);

    RTMPServer::new(
        listener,
        Box::new(ApiAuth::new(&auth_url)),
        Box::new(EventEmitter::new(addr).context("Failed to initialize event emitter")?),
    )
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

struct EventEmitter {
    client: redis::Client,
    rtmp_url: String,
}

impl EventEmitter {
    pub fn new(rtmp_url: &str) -> anyhow::Result<Self> {
        Ok(EventEmitter {
            client: redis::Client::open(env::var("REDIS_URL").context("REDIS_URL is missing")?)?,
            rtmp_url: rtmp_url.to_owned(),
        })
    }
}

#[async_trait]
impl StreamEventEmitter for EventEmitter {
    async fn on_published(&self, stream_id: &str) {
        let mut conn = match self.client.get_connection() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to establish connection with redis: {e}");
                return;
            }
        };

        let event = StreamEvent::Start {
            stream_id: stream_id.to_owned(),
            rtmp_url: self.rtmp_url.clone(),
        };
        let Ok(event_json) = serde_json::to_string(&event) else {
            error!("Failed to serialize event");
            return;
        };

        // i couldn't figure how to deal with the return type so we're using unwrap or else here
        let _: () = conn
            .xadd(STREAM_EVENT_KEY, "*", &[("data", event_json)])
            .unwrap_or_else(|_| error!("Failed to emit stream publish event"));
    }

    async fn on_stop(&self, stream_id: &str) {
        let Ok(mut conn) = self.client.get_connection() else {
            error!("Failed to establish connection with redis: {e}");
            return;
        };

        let event = StreamEvent::Stop {
            stream_id: stream_id.to_owned(),
        };
        let Ok(event_json) = serde_json::to_string(&event) else {
            error!("Failed to serialize event");
            return;
        };

        let _: () = conn
            .xadd(STREAM_EVENT_KEY, "*", &[("data", event_json)])
            .unwrap_or_else(|_| error!("Failed to emit stream stop event"));
    }
}
