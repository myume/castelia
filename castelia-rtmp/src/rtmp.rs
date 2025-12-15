use std::{collections::HashMap, io};

use bytes::Bytes;
use tokio::{
    io::BufReader,
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tracing::{Instrument, debug, error, info, instrument, trace};

use crate::{
    chunks::{Chunk, chunk_handler::ChunkHandler, header::MessageHeader},
    handshake::handshake,
    messages::{Message, router::MessageRouter},
};

pub struct RTMPSever {
    listener: TcpListener,
    streams: HashMap<String, tokio::sync::broadcast::Sender<Bytes>>,
}

impl RTMPSever {
    pub fn new(listener: TcpListener) -> Self {
        Self {
            listener,
            streams: HashMap::new(),
        }
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

pub(crate) type SendQueueMessage = (MessageHeader, Bytes);

struct RTMPConnection {
    socket: TcpStream,
    chunk_handler: ChunkHandler,
    message_router: MessageRouter,
}

impl RTMPConnection {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            chunk_handler: ChunkHandler::new(),
            message_router: MessageRouter::new(),
        }
    }

    async fn process(&mut self) -> io::Result<()> {
        handshake(&mut self.socket).await?;

        let (read_half, _write_half) = self.socket.split();

        let (sender, send_queue) = mpsc::channel(100);
        tokio::spawn(Self::send_pending_messages(send_queue).in_current_span());

        let mut reader = BufReader::new(read_half);
        loop {
            let max_chunk_size = self.message_router.net_connection().max_chunk_size() as usize;
            let chunk = Chunk::read_chunk(&mut reader, &max_chunk_size).await?;
            trace!("finished reading chunk");

            if let Some((message_bytes, message_type_id, message_stream_id)) =
                self.chunk_handler.receive_chunk(chunk)
            {
                match Message::parse_message(&message_bytes, message_type_id) {
                    Ok(msg) => {
                        info!("message received:\n{:#?}", msg);
                        self.message_router
                            .route_message(msg, message_stream_id, sender.clone())
                            .await;
                    }
                    Err(e) => error!("unable to parse message: {e}"),
                };
            }
        }
    }

    #[instrument(skip_all)]
    async fn send_pending_messages(mut send_queue: mpsc::Receiver<SendQueueMessage>) {
        info!("initialized outbound message processor");
        while let Some((message_header, payload)) = send_queue.recv().await {
            // chunk payload and send chunks
        }
    }
}
