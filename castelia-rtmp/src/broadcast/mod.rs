use std::fmt::Display;

use async_trait::async_trait;
use bytes::Bytes;
use thiserror::Error;

pub mod single_node;

pub struct SendError(String);

impl Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub enum SubscribeError {
    NotFound,
}

#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error("Stream is closed")]
    StreamClosed,
    #[error("Failure to receive message: {0}")]
    Other(String),
}

pub enum SetMetadataError {
    StreamNotFound,
}

#[derive(Debug, Clone)]
pub enum MediaType {
    Video,
    Audio,
}

#[async_trait]
pub trait Broadcaster: Send + Sync {
    async fn create_stream(&mut self, stream_key: &str) -> Box<dyn BroadcastStreamer>;

    async fn set_stream_video_header(
        &mut self,
        stream_key: &str,
        header: Bytes,
    ) -> Result<(), SetMetadataError>;

    async fn set_stream_audio_header(
        &mut self,
        stream_key: &str,
        header: Bytes,
    ) -> Result<(), SetMetadataError>;

    async fn set_stream_metadata(
        &mut self,
        stream_key: &str,
        metadata: Bytes,
    ) -> Result<(), SetMetadataError>;

    async fn delete_stream(&mut self, stream_key: &str);

    async fn subscribe(
        &self,
        channel_name: &str,
    ) -> Result<Box<dyn BroadcasterReceiver>, SubscribeError>;
}

#[async_trait]
pub trait BroadcastStreamer: Send + Sync {
    async fn send_data(&mut self, data: Bytes, media_type: MediaType) -> Result<(), SendError>;
}

#[async_trait]
pub trait BroadcasterReceiver: Send + Sync {
    async fn receive_video_header(&mut self) -> Result<Bytes, ReceiveError>;

    async fn receive_audio_header(&mut self) -> Result<Bytes, ReceiveError>;

    async fn receive_metadata(&mut self) -> Result<Bytes, ReceiveError>;

    async fn receive_data(&mut self) -> Result<(MediaType, Bytes), ReceiveError>;
}
