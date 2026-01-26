use std::{
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use castelia_events::stream_events::StreamEventEmitter;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tracing::{Instrument, debug, error, info, instrument, trace, warn};

use crate::{
    broadcast::{Broadcaster, authenticator::Authenticator, single_node::SingleNodeBroadcaster},
    chunks::{Chunk, SERVER_CHUNK_SIZE, chunk_handler::ChunkHandler, header::FullMessageHeader},
    handshake::handshake,
    messages::{
        Message,
        protocol_control::PeerBandwidth,
        router::{MessageRouter, RouteError},
    },
};

pub(crate) type Broadcasts = Arc<tokio::sync::Mutex<Box<dyn Broadcaster>>>;
pub(crate) type EventEmitter = Arc<tokio::sync::Mutex<Box<dyn StreamEventEmitter>>>;

pub struct RTMPServer {
    listener: TcpListener,
    broadcaster: Broadcasts,
    event_emitter: EventEmitter,
}

impl RTMPServer {
    pub fn new(
        listener: TcpListener,
        authenticator: Box<dyn Authenticator>,
        event_emitter: Box<dyn StreamEventEmitter>,
    ) -> Self {
        Self {
            listener,
            broadcaster: Arc::new(tokio::sync::Mutex::new(Box::new(
                SingleNodeBroadcaster::new(authenticator),
            ))),
            event_emitter: Arc::new(tokio::sync::Mutex::new(event_emitter)),
        }
    }

    pub async fn run(&self) -> io::Result<()> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            debug!("Accepted connection from {addr}");

            let broadcaster = self.broadcaster.clone();
            let event_emitter = self.event_emitter.clone();
            tokio::spawn(async move {
                handle_rtmp_connection(socket, addr, broadcaster, event_emitter).await;
            });
        }
    }
}

#[instrument(
    name = "RTMP connection",
    skip_all,
    fields(
        address =  addr.to_string()
    )
)]
async fn handle_rtmp_connection(
    socket: TcpStream,
    addr: SocketAddr,
    broadcaster: Broadcasts,
    event_emitter: EventEmitter,
) {
    let connection = RTMPConnection::new(broadcaster, event_emitter);
    if let Err(e) = connection.process(socket).await {
        match e.kind() {
            io::ErrorKind::UnexpectedEof
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionRefused => {
                debug!("Connection dropped by client")
            }
            _ => error!("Failed to process rtmp connection: {e}"),
        }
    }
}

pub(crate) type SendQueueMessage = (FullMessageHeader, Bytes);

pub(crate) struct RTMPConnectionState {
    pub max_chunk_size: u32,
    pub abort_queue: Vec<u32>,
    pub ack_seq_num: u32,
    pub peer_window_size: u32,
    pub server_window_size: u32,
    pub peer_bandwidth: PeerBandwidth,
}

impl RTMPConnectionState {
    pub fn new() -> Self {
        Self {
            max_chunk_size: 128,
            abort_queue: Vec::new(),
            ack_seq_num: 0,
            peer_window_size: 0,
            server_window_size: 2_500_000,
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
    chunk_handler: ChunkHandler,
    message_router: MessageRouter,
    connection_state: Arc<Mutex<RTMPConnectionState>>,
    broadcaster: Broadcasts,
    event_emitter: EventEmitter,
}

impl RTMPConnection {
    pub fn new(broadcaster: Broadcasts, event_emitter: EventEmitter) -> Self {
        Self {
            chunk_handler: ChunkHandler::new(),
            message_router: MessageRouter::new(),
            connection_state: Arc::new(Mutex::new(RTMPConnectionState::default())),
            broadcaster,
            event_emitter,
        }
    }

    async fn process(mut self, mut socket: TcpStream) -> io::Result<()> {
        handshake(&mut socket).await?;

        let (read_half, write_half) = socket.into_split();
        let (sender, send_queue) = mpsc::channel(100);
        tokio::spawn(
            Self::send_pending_messages(write_half, send_queue, self.connection_state.clone())
                .in_current_span(),
        );

        let mut reader = BufReader::with_capacity(65536, read_half);
        let mut should_exit = false;
        while !should_exit {
            let max_chunk_size = self
                .connection_state
                .lock()
                .map_err(|e| io::Error::other(e.to_string()))?
                .max_chunk_size as usize;

            let chunk = self
                .chunk_handler
                .read_chunk(&mut reader, &max_chunk_size)
                .await?;

            trace!("finished reading chunk");

            if let Some((message_bytes, message_type_id, message_stream_id, recent_header)) =
                self.chunk_handler.receive_chunk(chunk)
            {
                match Message::parse_message(&message_bytes, message_type_id) {
                    Ok(msg) => {
                        trace!("received message: {:#?}", msg);
                        if let Err(e) = self
                            .message_router
                            .route_message(
                                msg,
                                message_stream_id,
                                sender.clone(),
                                self.connection_state.clone(),
                                self.broadcaster.clone(),
                                &mut should_exit,
                                recent_header,
                                self.event_emitter.clone(),
                            )
                            .await
                        {
                            match e {
                                RouteError::HandleNetconnectonError(
                                    crate::netconnection::handler::HandleError::UnsupportedCommand(
                                        command,
                                    ),
                                ) => {
                                    warn!("Unsupported Command: {command}")
                                }
                                RouteError::HandleNetstreamError(
                                    crate::netstream::handler::HandleError::BroadcastNotFound,
                                ) => {
                                    debug!("Broadcast not found");
                                }
                                _ => error!("{e}"),
                            }
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

        info!("Closing connection");

        Ok(())
    }

    #[instrument(skip_all)]
    async fn send_pending_messages<T>(
        mut writer: T,
        mut send_queue: mpsc::Receiver<SendQueueMessage>,
        _connection_state: Arc<Mutex<RTMPConnectionState>>,
    ) where
        T: AsyncWriteExt + std::marker::Unpin,
    {
        debug!("Initialized outbound message processor");
        while let Some((message_header, payload)) = send_queue.recv().await {
            // chunk payload and send chunks
            let chunks = Chunk::into_chunks(&message_header, payload, SERVER_CHUNK_SIZE);
            trace!("chunked payload into {} chunks", chunks.len());
            for chunk in chunks {
                trace!("sending chunk:\n{:#?}", &chunk);
                if let Err(e) = writer.write_buf(&mut chunk.serialize()).await {
                    match e.kind() {
                        io::ErrorKind::BrokenPipe => {
                            debug!("Sending message to closed connection, dropping message.")
                        }
                        _ => error!("Failed to send message: {e}"),
                    }
                    return;
                }
            }
            debug!("sending message\n{:#?}", &message_header);
        }
    }
}
