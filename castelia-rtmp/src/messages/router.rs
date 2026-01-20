use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tracing::{info, trace};

use crate::{
    chunks::header::MessageHeader,
    messages::{Message, command::CommandMessage},
    netconnection::{self, NetConnection},
    netstream::{self, NetStream, command::NetStreamCommand},
    rtmp::{Broadcasts, RTMPConnectionState, SendQueueMessage},
};

#[derive(Error, Debug)]
pub enum RouteError {
    #[error("Message stream with id {0} does not exist")]
    MissingNetStream(u32),
    #[error("Netconnection commands must be issued on message stream 0, found {0}")]
    InvalidNetconnectionRoute(u32),
    #[error("Failed to handle netconnection command: {0}")]
    HandleNetconnectonError(
        #[source]
        #[from]
        netconnection::handler::HandleError,
    ),
    #[error("Failed to handle netstream command: {0}")]
    HandleNetstreamError(
        #[source]
        #[from]
        netstream::handler::HandleError,
    ),
}

#[derive(Default)]
pub struct MessageStream {
    message_streams: HashMap<u32, NetStream>,
}

impl MessageStream {
    pub fn get_mut(&mut self, msid: &u32) -> Option<&mut NetStream> {
        self.message_streams.get_mut(msid)
    }

    pub fn create_stream(&mut self) -> u32 {
        // We're going to use a naive implementation where we just keep appending new streams to
        // the hashmap which increments the stream id. This will be an issue if there's enough
        // streams to overflow a u32. We're going to assume that it's highly unlikely and dumb if
        // it happens.
        //
        // + 1 to skip message stream 0 (control stream)
        let message_stream_id = self.message_streams.len() as u32 + 1;
        self.message_streams
            .insert(message_stream_id, NetStream::new(message_stream_id));
        message_stream_id
    }

    pub fn delete_stream(&mut self, msid: &u32) -> Option<NetStream> {
        self.message_streams.remove(msid)
    }
}

pub struct MessageRouter {
    net_connection: NetConnection,
    message_streams: MessageStream,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            net_connection: NetConnection::new(),
            message_streams: MessageStream::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn route_message<'a>(
        &mut self,
        message: Message<'a>,
        message_stream_id: u32,
        send_queue: Sender<SendQueueMessage>,
        connection_state: Arc<Mutex<RTMPConnectionState>>,
        broadcaster: Broadcasts,
        should_exit: &mut bool,
        message_header: MessageHeader,
    ) -> Result<(), RouteError> {
        match message {
            Message::Protocol(protocol_control_message) => {
                trace!("routing message to netconnection");
                if message_stream_id != 0 {
                    return Err(RouteError::InvalidNetconnectionRoute(message_stream_id));
                }
                self.net_connection
                    .handle_protocol_message(protocol_control_message, connection_state)
                    .await
            }
            Message::UserControl(user_control_message) => {
                trace!("routing message to netconnection");
                if message_stream_id != 0 {
                    return Err(RouteError::InvalidNetconnectionRoute(message_stream_id));
                }
                self.net_connection
                    .handle_user_control_message(user_control_message, send_queue)
                    .await
            }
            Message::Command(command_message) => {
                if let CommandMessage::NetStreamCommand { ref command, .. } = command_message
                    && let NetStreamCommand::DeleteStream { stream_id } = command
                {
                    if let Some(stream) = self.message_streams.delete_stream(stream_id) {
                        if let Some(stream_id) = stream.stream_id {
                            broadcaster.lock().await.delete_stream(&stream_id).await;
                        }

                        info!("deleted stream {}", stream.id);
                    } else {
                        return Err(RouteError::MissingNetStream(*stream_id));
                    }
                    return Ok(());
                }

                if let CommandMessage::NetConnection(command) = command_message {
                    trace!("routing message to netconnection");
                    if message_stream_id != 0 {
                        return Err(RouteError::InvalidNetconnectionRoute(message_stream_id));
                    }
                    self.net_connection
                        .handle_command(
                            &message_stream_id,
                            command,
                            send_queue,
                            &mut self.message_streams,
                            connection_state,
                            should_exit,
                        )
                        .await?;
                } else {
                    trace!("routing message to netstream {message_stream_id}");
                    let Some(stream) = self.message_streams.get_mut(&message_stream_id) else {
                        return Err(RouteError::MissingNetStream(message_stream_id));
                    };

                    stream
                        .handle_message(command_message, send_queue, broadcaster, message_header)
                        .await?;
                }
            }
        }
        Ok(())
    }
}
