use std::io;

use thiserror::Error;
use tokio::{io::BufReader, net::TcpStream};

use crate::{
    chunks::header::{ChunkHeader, ParseChunkHeaderError},
    messages::RTMPMessage,
};

mod header;

pub struct Chunk {
    pub header: ChunkHeader,
    pub message: RTMPMessage,
}

#[derive(Error, Debug)]
pub enum ParseChunkError {
    #[error("Failed to parse header")]
    ParseHeaderError(
        #[source]
        #[from]
        ParseChunkHeaderError,
    ),
}

impl From<ParseChunkError> for io::Error {
    fn from(value: ParseChunkError) -> Self {
        match value {
            ParseChunkError::ParseHeaderError(parse_chunk_header_error) => {
                match parse_chunk_header_error {
                    ParseChunkHeaderError::ReadError(ref error) => {
                        io::Error::new(error.kind(), parse_chunk_header_error)
                    }
                    err => io::Error::new(io::ErrorKind::InvalidData, err),
                }
            }
        }
    }
}

impl Chunk {
    /// Read a Chunk from the stream
    pub async fn read_chunk(
        reader: &mut BufReader<&mut TcpStream>,
    ) -> Result<Self, ParseChunkError> {
        Ok(Self {
            header: ChunkHeader::read_header(reader).await?,
            message: RTMPMessage::read_message(reader).await,
        })
    }
}
