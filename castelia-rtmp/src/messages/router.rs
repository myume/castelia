use std::collections::HashMap;

use tokio::sync::mpsc::Sender;
use tracing::warn;

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
                self.net_connection
                    .handle_protocol_message(protocol_control_message, send_queue)
                    .await
            }
            Message::UserControl(user_control_message) => {
                self.net_connection
                    .handle_user_control_message(user_control_message, send_queue)
                    .await
            }
            Message::Command(command_message) => {
                if message_stream_id == 0 {
                    if let CommandMessage::NetConnectionCommand(command) = command_message {
                        // self.net_connection.handle_command();
                    } else {
                        warn!("");
                    }
                } else {
                    let stream = self.message_streams.get(&message_stream_id);
                }
            }
        }
    }
}
