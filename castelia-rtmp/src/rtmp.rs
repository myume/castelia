use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

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
    messages::{Message, protocol_control::PeerBandwidth, router::MessageRouter},
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

pub struct RTMPConnectionState {
    pub max_chunk_size: u32,
    pub abort_queue: Vec<u32>,
    pub ack_seq_num: u32,
    pub ack_window_size: u32,
    pub peer_bandwidth: PeerBandwidth,
}

impl RTMPConnectionState {
    pub fn new() -> Self {
        Self {
            max_chunk_size: 128,
            abort_queue: Vec::new(),
            ack_seq_num: 0,
            ack_window_size: 0,
            peer_bandwidth: PeerBandwidth {
                limit_type: 0,
                window_size: 0,
            },
        }
    }
}

impl Default for RTMPConnectionState {
    fn default() -> Self {
        Self::new()
    }
}

struct RTMPConnection {
    socket: TcpStream,
    chunk_handler: ChunkHandler,
    message_router: MessageRouter,
    connection_state: Arc<Mutex<RTMPConnectionState>>,
}

impl RTMPConnection {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            chunk_handler: ChunkHandler::new(),
            message_router: MessageRouter::new(),
            connection_state: Arc::new(Mutex::new(RTMPConnectionState::default())),
        }
    }

    async fn process(&mut self) -> io::Result<()> {
        handshake(&mut self.socket).await?;

        let (read_half, _write_half) = self.socket.split();

        let (sender, send_queue) = mpsc::channel(100);
        tokio::spawn(
            Self::send_pending_messages(send_queue, self.connection_state.clone())
                .in_current_span(),
        );

        let mut reader = BufReader::new(read_half);
        loop {
            let max_chunk_size = self
                .connection_state
                .lock()
                .map_err(|e| io::Error::other(e.to_string()))?
                .max_chunk_size as usize;

            let chunk = Chunk::read_chunk(&mut reader, &max_chunk_size).await?;
            trace!("finished reading chunk");

            if let Some((message_bytes, message_type_id, message_stream_id)) =
                self.chunk_handler.receive_chunk(chunk)
            {
                match Message::parse_message(&message_bytes, message_type_id) {
                    Ok(msg) => {
                        info!("message received:\n{:#?}", msg);
                        if let Err(e) = self
                            .message_router
                            .route_message(
                                msg,
                                message_stream_id,
                                sender.clone(),
                                self.connection_state.clone(),
                            )
                            .await
                        {
                            error!("failed to handle message: {e}");
                        }

                        let abort_queue = &mut self
                            .connection_state
                            .lock()
                            .map_err(|e| io::Error::other(e.to_string()))?
                            .abort_queue;

                        while let Some(chunk_to_abort) = abort_queue.pop() {
                            self.chunk_handler.abort(chunk_to_abort);
                        }
                    }
                    Err(e) => error!("unable to parse message: {e}"),
                };
            }
        }
    }

    #[instrument(skip_all)]
    async fn send_pending_messages(
        mut send_queue: mpsc::Receiver<SendQueueMessage>,
        _connection_state: Arc<Mutex<RTMPConnectionState>>,
    ) {
        info!("initialized outbound message processor");
        while let Some((message_header, payload)) = send_queue.recv().await {
            // chunk payload and send chunks
        }
    }
}
