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

const RTMP_VERSION: u8 = 0x03;

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("RTMP version provided by client is unsupported")]
    UnsupportedVersion,
    #[error("Failed to read from socket")]
    ReadError(#[source] io::Error),
    #[error("Failed to write to socket")]
    WriteError(#[source] io::Error),
    #[error("Invalid handshake: {0}")]
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
    read_c0(&mut socket).await?;
    trace!("Read C0");

    let mut client_buf = [0; HANDSHAKE_CHUNK_SIZE];
    let mut server_buf = [0; 1 + HANDSHAKE_CHUNK_SIZE];

    read_c1(&mut socket, &mut client_buf).await?;
    let read_timestamp = get_timestamp()?;
    trace!("Read C1");

    send_s0_s1(&mut socket, &mut server_buf).await?;
    trace!("Sent S0 and S1");

    send_s2(&mut socket, &mut client_buf, &read_timestamp).await?;
    trace!("Sent S2");

    // Read C2
    read_c2(
        &mut socket,
        server_buf[1..].try_into().map_err(|_| {
            // should never happen...
            HandshakeError::InvalidHandshake("Could not cast S1 into correct size".into())
        })?,
        &mut client_buf,
    )
    .await?;
    trace!("Read C2");

    Ok(())
}

async fn read_chunk(
    socket: &mut TcpStream,
    buf: &mut [u8; HANDSHAKE_CHUNK_SIZE],
) -> Result<(), HandshakeError> {
    let mut total_bytes_read = 0;
    while total_bytes_read < HANDSHAKE_CHUNK_SIZE {
        total_bytes_read += socket.read(buf).await.map_err(HandshakeError::ReadError)?;
    }

    Ok(())
}

async fn read_c0(socket: &mut TcpStream) -> Result<(), HandshakeError> {
    let version = socket.read_u8().await.map_err(HandshakeError::ReadError)?;
    trace!("RTMP version: {version}");
    if version != RTMP_VERSION {
        return Err(HandshakeError::UnsupportedVersion);
    }
    Ok(())
}

async fn read_c1(
    socket: &mut TcpStream,
    client_buf: &mut [u8; HANDSHAKE_CHUNK_SIZE],
) -> Result<(), HandshakeError> {
    read_chunk(socket, client_buf).await?;
    let zeroes = &client_buf[4..8];
    if !zeroes.iter().all(|x| *x == 0) {
        return Err(HandshakeError::InvalidHandshake(
            "Zeroes field in handshake must be all zeroes".into(),
        ));
    }

    Ok(())
}

async fn send_s0_s1(
    socket: &mut TcpStream,
    server_buf: &mut [u8; 1 + HANDSHAKE_CHUNK_SIZE],
) -> Result<(), HandshakeError> {
    // send version along
    server_buf[0] = RTMP_VERSION;

    // timestamp
    server_buf[1..5].copy_from_slice(&get_timestamp()?);

    // random data
    for i in &mut server_buf[9..] {
        *i = rand::random();
    }

    socket
        .write_all(server_buf)
        .await
        .map_err(HandshakeError::WriteError)
}

async fn send_s2(
    socket: &mut TcpStream,
    c1: &mut [u8; HANDSHAKE_CHUNK_SIZE],
    read_timestamp: &[u8; 4],
) -> Result<(), HandshakeError> {
    c1[4..8].copy_from_slice(read_timestamp);
    socket
        .write_all(c1)
        .await
        .map_err(HandshakeError::WriteError)?;

    Ok(())
}

async fn read_c2(
    socket: &mut TcpStream,
    s1: &[u8; HANDSHAKE_CHUNK_SIZE],
    client_buf: &mut [u8; HANDSHAKE_CHUNK_SIZE],
) -> Result<(), HandshakeError> {
    read_chunk(socket, client_buf).await?;
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

    Ok(())
}

fn get_timestamp() -> Result<[u8; 4], HandshakeError> {
    let bytes = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| {
            HandshakeError::InvalidHandshake(
                "Could not generate timestamp for handshake, clock may have gone backwards".into(),
            )
        })?
        .as_millis()
        .to_be_bytes();

    // u128 is 16 bytes, it is safe to take the last 4
    Ok([bytes[12], bytes[13], bytes[14], bytes[15]])
}
