use std::collections::HashMap;

use bytes::{Bytes, BytesMut};
use tracing::error;

use crate::chunks::{CSId, Chunk};

#[derive(Debug)]
struct PartialMessage {
    length: u32,
    message_type: u8,
    bytes: BytesMut,
}

/// Receives chunks and multiplexes it to the correct chunk stream
#[derive(Debug)]
pub struct ChunkMultiplexer {
    chunk_streams: HashMap<CSId, PartialMessage>,
}

impl ChunkMultiplexer {
    pub fn receive_chunk(&mut self, chunk: Chunk) -> Option<(Bytes, u8)> {
        let cs_id = chunk.header.chunk_stream_id();
        if let Some(partial) = self.chunk_streams.get_mut(&cs_id) {
            partial.bytes.extend(chunk.payload);
        } else if let Some(length) = chunk.header.get_message_length()
            && let Some(message_type) = chunk.header.get_message_type()
        {
            self.chunk_streams.insert(
                cs_id,
                PartialMessage {
                    length,
                    message_type,
                    bytes: chunk.payload.into(),
                },
            );
        } else {
            error!("Incomplete message header, dropping chunk");
            return None;
        }

        if let Some(partial) = self.chunk_streams.get(&cs_id)
            && partial.length as usize == partial.bytes.len()
            && let Some(partial) = self.chunk_streams.remove(&cs_id)
        {
            Some((partial.bytes.into(), partial.message_type))
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
