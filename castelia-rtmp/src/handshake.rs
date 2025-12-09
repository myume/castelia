use std::{
    io,
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
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
    #[error("{0}")]
    InvalidHandshake(String),
}

impl From<HandshakeError> for io::Error {
    fn from(value: HandshakeError) -> Self {
        match value {
            HandshakeError::UnsupportedVersion => io::Error::new(io::ErrorKind::Unsupported, value),
            HandshakeError::InvalidHandshake(s) => io::Error::new(io::ErrorKind::InvalidData, s),
            HandshakeError::ReadError(ref error) => io::Error::new(error.kind(), value),
            HandshakeError::WriteError(ref error) => io::Error::new(error.kind(), value),
        }
    }
}

/// Performs a RTMP handshake on the provided socket
pub async fn handshake(mut socket: TcpStream) -> Result<(), HandshakeError> {
    // Read C0
    let version = socket.read_u8().await.map_err(HandshakeError::ReadError)?;
    trace!("RTMP version: {version}");
    if version != 3 {
        return Err(HandshakeError::UnsupportedVersion);
    }

    let mut client_buf = [0; HANDSHAKE_CHUNK_SIZE];

    // Read C1
    read_chunk(&mut socket, &mut client_buf).await?;
    let read_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| {
        HandshakeError::InvalidHandshake("Could not generate timestamp for handshake".into())
    })?;
    trace!("Read C1");
    let zeroes = &client_buf[4..8];
    if !zeroes.iter().all(|x| *x == 0) {
        return Err(HandshakeError::InvalidHandshake(
            "Zeroes field in handshake must be all zeroes".into(),
        ));
    }

    // Send S0 and S1
    let mut server_buf = [0; 1 + HANDSHAKE_CHUNK_SIZE];
    server_buf[0] = 0x03;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| {
        HandshakeError::InvalidHandshake("Could not generate timestamp for handshake".into())
    })?;
    server_buf[1..5].copy_from_slice(&timestamp.as_secs().to_be_bytes()[4..8]);
    for i in &mut server_buf[9..] {
        *i = rand::random();
    }

    socket
        .write_all(&server_buf)
        .await
        .map_err(HandshakeError::WriteError)?;
    trace!("Sent S0 and S1");

    // Send S2 (echo C1)
    client_buf[4..8].copy_from_slice(&read_timestamp.as_secs().to_be_bytes()[4..8]);
    socket
        .write_all(&client_buf)
        .await
        .map_err(HandshakeError::WriteError)?;
    trace!("Sent S2");

    // Read C2
    read_chunk(&mut socket, &mut client_buf).await?;
    trace!("Read C2");
    let s1 = &server_buf[1..];
    if client_buf[..4] != s1[..4] {
        return Err(HandshakeError::InvalidHandshake(
            "Echoed timestamp does not match".into(),
        ));
    }

    if client_buf[8..] != s1[8..] {
        return Err(HandshakeError::InvalidHandshake(
            "Random echo does not match".into(),
        ));
    }
    trace!("Verified echo");

    Ok(())
}

async fn read_chunk(socket: &mut TcpStream, buf: &mut [u8]) -> Result<(), HandshakeError> {
    let mut total_bytes_read = 0;
    while total_bytes_read < HANDSHAKE_CHUNK_SIZE {
        total_bytes_read += socket.read(buf).await.map_err(HandshakeError::ReadError)?;
    }

    Ok(())
}
