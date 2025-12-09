use std::io;

use thiserror::Error;
use tokio::{io::AsyncReadExt, net::TcpStream};
use tracing::trace;

/// The size of the C1/C2/S1/S2 chunks:
///
/// C1/S1 chunks consist of:
/// - 4 byte time field
/// - 4 byte zeroes
/// - 1528 bytes of random data
///
/// C2/S2 chunks consist of:
/// - 4 byte time field
/// - 4 byte time 2 field
/// - 1528 byte echo field
const HANDSHAKE_CHUNK_SIZE: usize = 4 + 4 + 1528;

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("RTMP version provided by client is unsupported")]
    UnsupportedVersion,
    #[error("Failed to read from socket")]
    ReadError(#[source] io::Error),
    #[error("Failed to write to socket")]
    WriteError(#[source] io::Error),
}

impl From<HandshakeError> for io::Error {
    fn from(value: HandshakeError) -> Self {
        match value {
            HandshakeError::UnsupportedVersion => io::Error::new(io::ErrorKind::Unsupported, value),
            HandshakeError::ReadError(ref error) => io::Error::new(error.kind(), value),
            HandshakeError::WriteError(ref error) => io::Error::new(error.kind(), value),
        }
    }
}

/// Performs a RTMP handshake on the provided socket
pub async fn handshake(mut socket: TcpStream) -> Result<(), HandshakeError> {
    let version = socket.read_u8().await.map_err(HandshakeError::ReadError)?;
    trace!("RTMP version: {version}");
    if version != 3 {
        return Err(HandshakeError::UnsupportedVersion);
    }

    let mut buf = [0; HANDSHAKE_CHUNK_SIZE];

    // read C1
    read_handshake_chunk(socket, &mut buf).await?;
    trace!("C1: {:#04x?}", &buf);

    Ok(())
}

async fn read_handshake_chunk(mut socket: TcpStream, buf: &mut [u8]) -> Result<(), HandshakeError> {
    let mut total_bytes_read = 0;
    while total_bytes_read < HANDSHAKE_CHUNK_SIZE {
        total_bytes_read += socket.read(buf).await.map_err(HandshakeError::ReadError)?;
    }

    Ok(())
}
