use std::io;

use tokio::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};
use tracing::{debug, error, instrument, trace};

use crate::{
    chunks::{Chunk, chunk_mux::ChunkMultiplexer},
    handshake::handshake,
    messages::Message,
};

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
    max_chunk_size: usize,
    chunk_mux: ChunkMultiplexer,
}

impl RTMPConnection {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            max_chunk_size: 4096,
            chunk_mux: ChunkMultiplexer::new(),
        }
    }

    async fn process(&mut self) -> io::Result<()> {
        handshake(&mut self.socket).await?;

        let mut reader = BufReader::new(&mut self.socket);
        loop {
            let chunk = Chunk::read_chunk(&mut reader, &self.max_chunk_size).await?;
            trace!("finished reading chunk");

            if let Some((message_bytes, message_type_id)) = self.chunk_mux.receive_chunk(chunk) {
                match Message::parse_message(&message_bytes, message_type_id) {
                    Ok(msg) => debug!("message received:\n{:#?}", msg),
                    Err(e) => error!("unable to parse message: {e}"),
                };
            }
        }
    }
}
