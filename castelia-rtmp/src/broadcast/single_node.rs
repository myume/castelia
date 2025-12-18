use std::collections::HashMap;

use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::broadcast::{
    BroadcastStreamer, Broadcaster, BroadcasterReceiver, MediaType, ReceiveError, SendError,
    SetMetadataError, SubscribeError,
};

pub struct SingleNodeBroadcaster {
    streams: HashMap<String, Stream>,
}

struct NaiveReceiver {
    data: Bytes,
    receiver: Receiver<(MediaType, Bytes)>,
}

struct Stream {
    metadata: Option<Bytes>,
    sender: Sender<(MediaType, Bytes)>,
}

impl SingleNodeBroadcaster {
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
        }
    }
}

#[async_trait]
impl Broadcaster for SingleNodeBroadcaster {
    async fn create_stream(&mut self, stream_key: &str) -> Box<dyn BroadcastStreamer> {
        let (tx, _) = broadcast::channel(100);
        let stream = Stream {
            metadata: None,
            sender: tx.clone(),
        };
        self.streams.insert(stream_key.to_owned(), stream);
        Box::new(tx)
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

    async fn delete_stream(&self, stream_key: &str) {
        todo!()
    }

    async fn subscribe(
        &self,
        channel_name: &str,
    ) -> Result<Box<dyn BroadcasterReceiver>, SubscribeError> {
        let stream = self
            .streams
            .get(channel_name)
            .ok_or(SubscribeError::NotFound(format!(
                "Channel with name {channel_name} not found"
            )))?;

        let Some(metadata) = &stream.metadata else {
            return Err(SubscribeError::NotFound(format!(
                "Channel with name {channel_name} not found"
            )));
        };

        Ok(Box::new(NaiveReceiver {
            data: metadata.clone(),
            receiver: stream.sender.subscribe(),
        }))
    }
}

#[async_trait]
impl BroadcastStreamer for Sender<(MediaType, Bytes)> {
    async fn send_data(&mut self, data: Bytes, media_type: MediaType) -> Result<(), SendError> {
        if self.receiver_count() > 0 {
            self.send((media_type, data))
                .map_err(|e| SendError(format!("Failed to send data to stream: {e}")))?;
        }
        Ok(())
    }
}

#[async_trait]
impl BroadcasterReceiver for NaiveReceiver {
    async fn receive_metadata(&mut self) -> Result<Bytes, ReceiveError> {
        Ok(self.data.clone())
    }

    async fn receive_data(&mut self) -> Result<(MediaType, Bytes), ReceiveError> {
        Ok(self.receiver.recv().await.map_err(|err| match err {
            broadcast::error::RecvError::Closed => ReceiveError::StreamClosed,
            broadcast::error::RecvError::Lagged(_) => {
                ReceiveError::Other(format!("Failed to receive data: {err}"))
            }
        })?)
    }
}
