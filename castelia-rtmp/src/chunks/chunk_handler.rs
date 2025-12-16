use std::collections::HashMap;

use bytes::{Bytes, BytesMut};
use tracing::{error, warn};

use crate::chunks::{CSId, Chunk};

#[derive(Debug)]
struct PartialMessage {
    length: u32,
    message_type: u8,
    message_stream_id: u32,
    bytes: BytesMut,
}

#[derive(Debug)]
pub struct ChunkHandler {
    chunk_streams: HashMap<CSId, PartialMessage>,
}

impl ChunkHandler {
    pub fn receive_chunk(&mut self, chunk: Chunk) -> Option<(Bytes, u8, u32)> {
        let cs_id = chunk.header.chunk_stream_id();
        if let Some(partial) = self.chunk_streams.get_mut(&cs_id) {
            if let Some(message_length) = chunk.header.get_message_length() {
                partial.length = message_length;
            }
            if let Some(message_type) = chunk.header.get_message_type() {
                partial.message_type = message_type;
            }
            if let Some(message_stream_id) = chunk.header.get_message_stream_id() {
                partial.message_stream_id = message_stream_id;
            }
            partial.bytes.extend(chunk.payload);
        } else if let Some(length) = chunk.header.get_message_length()
            && let Some(message_type) = chunk.header.get_message_type()
            && let Some(message_stream_id) = chunk.header.get_message_stream_id()
        {
            self.chunk_streams.insert(
                cs_id,
                PartialMessage {
                    length,
                    message_type,
                    message_stream_id,
                    bytes: chunk.payload.into(),
                },
            );
        } else {
            error!("Incomplete message header, dropping chunk");
            return None;
        }

        if let Some(partial) = self.chunk_streams.get_mut(&cs_id)
            && partial.length as usize == partial.bytes.len()
        {
            let message = (
                partial.bytes.clone().into(),
                partial.message_type,
                partial.message_stream_id,
            );
            partial.bytes.clear();
            Some(message)
        } else {
            None
        }
    }

    pub fn abort(&mut self, chunk_stream_id: CSId) {
        if self.chunk_streams.remove(&chunk_stream_id).is_none() {
            warn!("Aborting nonexisitent chunk stream {chunk_stream_id}");
        }
    }

    pub fn new() -> Self {
        Self {
            chunk_streams: HashMap::new(),
        }
    }
}
