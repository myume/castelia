use std::{collections::HashMap, io, time::Duration};

use bytes::{Bytes, BytesMut};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, BufReader},
    time::timeout,
};
use tracing::{debug, error, trace, warn};

use crate::chunks::{
    CSId, Chunk,
    header::{ChunkHeader, ParseChunkHeaderError},
};

#[derive(Error, Debug)]
pub enum ParseChunkError {
    #[error("Failed to parse header")]
    BadHeader(
        #[source]
        #[from]
        ParseChunkHeaderError,
    ),
    #[error("Connection timed out")]
    Timeout(
        #[source]
        #[from]
        tokio::time::error::Elapsed,
    ),
    #[error("Failed to read message")]
    MessageReadFailure(#[from] tokio::io::Error),
    #[error("Unknown message length")]
    UnknownMessageLength,
}

impl From<ParseChunkError> for io::Error {
    fn from(value: ParseChunkError) -> Self {
        match value {
            ParseChunkError::BadHeader(parse_chunk_header_error) => {
                match parse_chunk_header_error {
                    ParseChunkHeaderError::ReadError(ref error) => {
                        io::Error::new(error.kind(), parse_chunk_header_error)
                    }
                    err => io::Error::new(io::ErrorKind::InvalidData, err),
                }
            }
            ParseChunkError::Timeout(elapsed) => elapsed.into(),
            ParseChunkError::MessageReadFailure(ref error) => io::Error::new(error.kind(), value),
            ParseChunkError::UnknownMessageLength => io::Error::other(value.to_string()),
        }
    }
}

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
    /// Read a Chunk from the stream
    pub async fn read_chunk<T>(
        &self,
        reader: &mut BufReader<T>,
        max_chunk_size: &usize,
    ) -> Result<Chunk, ParseChunkError>
    where
        T: AsyncReadExt + std::marker::Unpin,
    {
        let header = timeout(Duration::from_secs(30), ChunkHeader::read_header(reader)).await??;
        debug!("chunk header has been parsed:\n{:#?}", header);

        let remaining_message_length = self
            .chunk_streams
            .get(&header.chunk_stream_id())
            .map(|partial_message| partial_message.length - partial_message.bytes.len() as u32);

        let message_length = header
            .get_message_length()
            .or(remaining_message_length)
            .ok_or(ParseChunkError::UnknownMessageLength)?;

        let payload_size = (*max_chunk_size).min(message_length as usize);

        let mut payload = BytesMut::zeroed(payload_size);
        timeout(Duration::from_secs(30), reader.read_exact(&mut payload)).await??;
        trace!("message read {:?}", &payload);

        Ok(Chunk {
            header,
            payload: payload.into(),
        })
    }

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
