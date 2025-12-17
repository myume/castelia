use async_trait::async_trait;
use bytes::Bytes;

pub mod naive;

pub struct SendError(String);

pub enum SubscribeError {
    NotFound(String),
}

pub enum ReceiveError {
    StreamClosed,
    Other(String),
}

#[async_trait]
pub trait Broadcaster: Send + Sync {
    async fn create_stream(&mut self, stream_key: &str) -> Box<dyn BroadcastStreamer>;

    async fn delete_stream(&self, stream_key: &str);

    async fn subscribe(
        &self,
        channel_name: &str,
    ) -> Result<Box<dyn BroadcasterReceiver>, SubscribeError>;
}

#[async_trait]
pub trait BroadcastStreamer: Send + Sync {
    async fn send_data(&self, data: Bytes) -> Result<(), SendError>;
}

#[async_trait]
pub trait BroadcasterReceiver: Send + Sync {
    async fn receive_metadata(&mut self) -> Result<Bytes, ReceiveError>;

    async fn receive_data(&mut self) -> Result<Bytes, ReceiveError>;
}
