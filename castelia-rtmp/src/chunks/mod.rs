use std::{io, time::Duration};

use bytes::{Bytes, BytesMut};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, BufReader},
    net::TcpStream,
    time::timeout,
};
use tracing::trace;

use crate::chunks::header::{ChunkHeader, ParseChunkHeaderError};

pub mod chunk_mux;
mod header;

type CSId = u32;

pub struct Chunk {
    pub header: ChunkHeader,
    pub payload: Bytes,
}

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
        }
    }
}

impl Chunk {
    /// Read a Chunk from the stream
    pub async fn read_chunk(
        reader: &mut BufReader<&mut TcpStream>,
        max_chunk_size: &usize,
    ) -> Result<Self, ParseChunkError> {
        let header = timeout(Duration::from_secs(30), ChunkHeader::read_header(reader)).await??;
        trace!("chunk header has been parsed:\n{:#?}", header);

        let max_bytes_remaining = max_chunk_size - header.len();
        let payload_size =
            max_bytes_remaining.min(header.get_message_length().unwrap_or(0) as usize);

        let mut payload = BytesMut::zeroed(payload_size);
        reader.read_exact(&mut payload).await?;
        trace!("message read {:?}", &payload);

        Ok(Self {
            header,
            payload: payload.into(),
        })
    }
}
