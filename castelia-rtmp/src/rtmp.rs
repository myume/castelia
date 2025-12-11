use std::io;

use tokio::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};
use tracing::{debug, error, instrument, trace};

use crate::{chunks::Chunk, handshake::handshake};

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

            let mut connection = RTMPConnection::new(socket);
            tokio::spawn(async move {
                if let Err(e) = connection.process().await {
                    error!("Failed to process rtmp connection: {e}");
                }
            });
        }
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

    #[instrument(
        name = "RTMP connection",
        skip_all,
        fields(
            address = %self
                        .socket
                        .peer_addr()
                        .map(|addr| addr.to_string())
                        .unwrap_or("unknown address".to_owned())
        )
    )]
    async fn process(&mut self) -> io::Result<()> {
        handshake(&mut self.socket).await?;

        let mut reader = BufReader::new(&mut self.socket);
        loop {
            Chunk::read_chunk(&mut reader).await?;
        }
    }
}
