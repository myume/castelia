use std::{io, time::Duration};

use tokio::{
    io::{AsyncReadExt, BufReader},
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tracing::{debug, error, instrument, trace};

use crate::{chunks::header::ChunkHeader, handshake::handshake};

pub struct RTMPSever {
    listener: TcpListener,
}

impl RTMPSever {
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    pub async fn run(&self) -> io::Result<()> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            debug!("Accepted connection from {addr}");

            tokio::spawn(async move {
                handle_rtmp_connection(RTMPConnection::new(socket)).await;
            });
        }
    }
}

#[instrument(
    name = "RTMP connection",
    skip_all,
    fields(
        address = connection
                    .socket
                    .peer_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or("unknown address".to_owned())
    )
)]
async fn handle_rtmp_connection(mut connection: RTMPConnection) {
    if let Err(e) = connection.process().await {
        error!("Failed to process rtmp connection: {e}");
    }
}

#[derive(Debug)]
struct RTMPConnection {
    socket: TcpStream,
}

impl RTMPConnection {
    pub fn new(socket: TcpStream) -> Self {
        Self { socket }
    }

    async fn process(&mut self) -> io::Result<()> {
        handshake(&mut self.socket).await?;

        let mut reader = BufReader::new(&mut self.socket);
        loop {
            let chunk_header = timeout(
                Duration::from_secs(30),
                ChunkHeader::read_header(&mut reader),
            )
            .await??; // the forbidden double question mark
            trace!("chunk header has been parsed:\n{:#?}", chunk_header);
            let message_length = chunk_header.get_message_length().unwrap_or(128);
            let mut buf = Vec::with_capacity(message_length as usize);
            let bytes_read = reader.read_buf(&mut buf).await?;

            dbg!(bytes_read, String::from_utf8_lossy(&buf[..bytes_read]));
        }
    }
}
