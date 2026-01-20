use std::collections::HashMap;

use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, error};

use crate::{
    broadcast::{
        BroadcastStreamer, Broadcaster, BroadcasterReceiver, CreateStreamError, MediaType, Payload,
        ReceiveError, SendError, SetMetadataError, SubscribeError, authenticator::Authenticator,
    },
    chunks::header::MessageHeader,
};

pub struct SingleNodeBroadcaster {
    authenticator: Box<dyn Authenticator>,
    streams: HashMap<String, Stream>,
}

struct StreamReceiver {
    metadata: Bytes,
    audio_header: Bytes,
    video_header: Bytes,
    receiver: Receiver<Payload>,
}

struct Stream {
    metadata: Option<Bytes>,
    audio_header: Option<Bytes>,
    video_header: Option<Bytes>,
    sender: Sender<Payload>,
}

#[async_trait]
impl Broadcaster for SingleNodeBroadcaster {
    fn new(authenticator: Box<dyn Authenticator>) -> Self {
        Self {
            streams: HashMap::new(),
            authenticator,
        }
    }

    async fn create_stream(
        &mut self,
        stream_key: &str,
    ) -> Result<(Box<dyn BroadcastStreamer>, String), CreateStreamError> {
        let (tx, _) = broadcast::channel(128); // arbitrary number, figure out what this should be
        let stream_id = self.authenticator.authenticate(stream_key).await?;
        let stream = Stream {
            metadata: None,
            sender: tx.clone(),
            audio_header: None,
            video_header: None,
        };
        self.streams.insert(stream_id.clone(), stream);
        Ok((Box::new(tx), stream_id))
    }

    async fn set_stream_video_header(
        &mut self,
        stream_key: &str,
        header: Bytes,
    ) -> Result<(), SetMetadataError> {
        let Some(stream) = self.streams.get_mut(stream_key) else {
            return Err(SetMetadataError::StreamNotFound);
        };

        stream.video_header = Some(header);
        debug!("Video header set");
        Ok(())
    }

    async fn set_stream_audio_header(
        &mut self,
        stream_key: &str,
        header: Bytes,
    ) -> Result<(), SetMetadataError> {
        let Some(stream) = self.streams.get_mut(stream_key) else {
            return Err(SetMetadataError::StreamNotFound);
        };

        stream.audio_header = Some(header);
        debug!("Audio header set");
        Ok(())
    }

    async fn set_stream_metadata(
        &mut self,
        stream_key: &str,
        metadata: Bytes,
    ) -> Result<(), SetMetadataError> {
        let Some(stream) = self.streams.get_mut(stream_key) else {
            return Err(SetMetadataError::StreamNotFound);
        };

        stream.metadata = Some(metadata);
        Ok(())
    }

    async fn delete_stream(&mut self, stream_id: &str) {
        self.streams.remove(stream_id);
    }

    async fn subscribe(
        &self,
        channel_name: &str,
    ) -> Result<Box<dyn BroadcasterReceiver>, SubscribeError> {
        let stream = self
            .streams
            .get(channel_name)
            .ok_or(SubscribeError::NotFound)?;

        let Some(metadata) = &stream.metadata else {
            error!("Stream metadata not found");
            return Err(SubscribeError::NotFound);
        };

        let Some(video_header) = &stream.video_header else {
            error!("Stream video header not found");
            return Err(SubscribeError::NotFound);
        };

        let Some(audio_header) = &stream.audio_header else {
            error!("Stream audio header not found");
            return Err(SubscribeError::NotFound);
        };

        Ok(Box::new(StreamReceiver {
            metadata: metadata.clone(),
            receiver: stream.sender.subscribe(),
            video_header: video_header.clone(),
            audio_header: audio_header.clone(),
        }))
    }
}

#[async_trait]
impl BroadcastStreamer for Sender<Payload> {
    async fn send_data(
        &mut self,
        data: Bytes,
        media_type: MediaType,
        message_header: MessageHeader,
    ) -> Result<(), SendError> {
        if self.receiver_count() > 0 {
            self.send((media_type, message_header, data))
                .map_err(|e| SendError(format!("Failed to send data to stream: {e}")))?;
        }
        Ok(())
    }
}

#[async_trait]
impl BroadcasterReceiver for StreamReceiver {
    async fn receive_video_header(&mut self) -> Result<Bytes, ReceiveError> {
        Ok(self.video_header.clone())
    }

    async fn receive_audio_header(&mut self) -> Result<Bytes, ReceiveError> {
        Ok(self.audio_header.clone())
    }

    async fn receive_metadata(&mut self) -> Result<Bytes, ReceiveError> {
        Ok(self.metadata.clone())
    }

    async fn receive_data(&mut self) -> Result<Payload, ReceiveError> {
        Ok(self.receiver.recv().await.map_err(|err| match err {
            broadcast::error::RecvError::Closed => ReceiveError::StreamClosed,
            broadcast::error::RecvError::Lagged(_) => {
                ReceiveError::Other(format!("Failed to receive data: {err}"))
            }
        })?)
    }
}
