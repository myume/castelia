use std::collections::HashMap;

use tokio::sync::mpsc::Sender;
use tracing::{trace, warn};

use crate::{
    messages::{Message, command::CommandMessage},
    netconnection::NetConnection,
    netstream::NetStream,
    rtmp::SendQueueMessage,
};

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

    pub fn net_connection(&self) -> &NetConnection {
        &self.net_connection
    }

    pub async fn route_message<'a>(
        &mut self,
        message: Message<'a>,
        message_stream_id: u32,
        send_queue: Sender<SendQueueMessage>,
    ) {
        match message {
            Message::Protocol(protocol_control_message) => {
                trace!("routing message to netconnection");
                self.net_connection
                    .handle_protocol_message(protocol_control_message, send_queue)
                    .await
            }
            Message::UserControl(user_control_message) => {
                trace!("routing message to netconnection");
                self.net_connection
                    .handle_user_control_message(user_control_message, send_queue)
                    .await
            }
            Message::Command(command_message) => {
                if message_stream_id == 0 {
                    if let CommandMessage::NetConnection(command) = command_message {
                        trace!("routing message to netconnection");
                        self.net_connection
                            .handle_command(command, send_queue)
                            .await;
                    } else {
                        warn!("netconnection could not handle command: {command_message:?}");
                    }
                } else {
                    trace!("routing message to netstream {message_stream_id}");
                    let stream = self.message_streams.get(&message_stream_id);
                }
            }
        }
    }
}
