use std::collections::HashMap;

use bytes::{Bytes, BytesMut};
use tracing::error;

use crate::chunks::{CSId, Chunk};

#[derive(Debug)]
struct PartialMessage {
    length: u32,
    bytes: BytesMut,
}

/// Receives chunks and multiplexes it to the correct chunk stream
#[derive(Debug)]
pub struct ChunkMultiplexer {
    chunk_streams: HashMap<CSId, PartialMessage>,
}

impl ChunkMultiplexer {
    pub fn receive_chunk(&mut self, chunk: Chunk) -> Option<Bytes> {
        let cs_id = chunk.header.chunk_stream_id();
        if let Some(partial) = self.chunk_streams.get_mut(&cs_id) {
            partial.bytes.extend(chunk.payload);
        } else if let Some(length) = chunk.header.get_message_length() {
            self.chunk_streams.insert(
                cs_id,
                PartialMessage {
                    length,
                    bytes: chunk.payload.into(),
                },
            );
        } else {
            error!("Missing message length, dropping chunk");
            return None;
        }

        if let Some(partial) = self.chunk_streams.get(&cs_id)
            && partial.length as usize == partial.bytes.len()
            && let Some(partial) = self.chunk_streams.remove(&cs_id)
        {
            Some(partial.bytes.into())
        } else {
            None
        }
    }

    pub fn new() -> Self {
        Self {
            chunk_streams: HashMap::new(),
        }
    }
}
