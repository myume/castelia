use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tracing::trace;

use crate::{
    messages::{Message, command::CommandMessage},
    netconnection::NetConnection,
    netstream::NetStream,
    rtmp::{RTMPConnectionState, SendQueueMessage},
};

#[derive(Error, Debug)]
pub enum RouteError {
    #[error("Message was routed to message stream {0}, which does not exist")]
    MissingNetStream(u32),
    #[error("Netconnection commands must be issued on message stream 0, found {0}")]
    InvalidNetconnectionRoute(u32),
}

pub struct MessageRouter {
    net_connection: NetConnection,
    message_streams: HashMap<u32, NetStream>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            net_connection: NetConnection::new(),
            message_streams: HashMap::new(),
        }
    }

    pub async fn route_message<'a>(
        &mut self,
        message: Message<'a>,
        message_stream_id: u32,
        send_queue: Sender<SendQueueMessage>,
        connection_state: Arc<Mutex<RTMPConnectionState>>,
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
                if let CommandMessage::NetConnection(command) = command_message {
                    trace!("routing message to netconnection");
                    if message_stream_id != 0 {
                        return Err(RouteError::InvalidNetconnectionRoute(message_stream_id));
                    }
                    self.net_connection
                        .handle_command(command, send_queue)
                        .await;
                } else {
                    trace!("routing message to netstream {message_stream_id}");
                    let Some(stream) = self.message_streams.get(&message_stream_id) else {
                        return Err(RouteError::MissingNetStream(message_stream_id));
                    };
                }
            }
        }
        Ok(())
    }
}
