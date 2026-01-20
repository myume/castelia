use std::collections::HashMap;

use async_trait::async_trait;
use castelia_rtmp::{
    broadcast::authenticator::{AuthenticateError, Authenticator},
    rtmp::RTMPServer,
};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1935").await?;
    info!("Listening on {}", listener.local_addr()?);

    let authenticator = TestAutheticator {
        key_map: HashMap::from([("test".into(), "test_user".into())]),
    };
    RTMPServer::new(listener, Box::new(authenticator))
        .run()
        .await?;

    Ok(())
}

struct TestAutheticator {
    key_map: HashMap<String, String>,
}

#[async_trait]
impl Authenticator for TestAutheticator {
    async fn authenticate(&self, stream_key: &str) -> Result<String, AuthenticateError> {
        self.key_map
            .get(stream_key)
            .cloned()
            .ok_or(AuthenticateError::InvalidKey)
    }
}
