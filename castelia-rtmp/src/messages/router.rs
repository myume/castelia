use tokio::sync::mpsc::Sender;

use crate::{messages::Message, netconnection::NetConnection, rtmp::SendQueueMessage};

#[derive(Debug)]
pub struct MessageRouter {
    net_connection: NetConnection,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            net_connection: NetConnection::new(),
        }
    }

    pub fn net_connection(&self) -> &NetConnection {
        &self.net_connection
    }

    pub async fn route_message<'a>(
        &mut self,
        message: Message<'a>,
        _message_stream_id: u32,
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
            Message::Command(command_message) => todo!(),
        }
    }
}
