use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub const STREAM_EVENT_KEY: &str = "stream:event";

#[async_trait]
pub trait StreamEventEmitter: Send + Sync {
    async fn on_published(&self, stream_id: &str);
    async fn on_stop(&self, stream_id: &str);
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StreamEvent {
    Start { stream_id: String, rtmp_url: String },
    Stop { stream_id: String },
}
