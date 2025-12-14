use bytes::Bytes;
use tokio::sync::mpsc::Sender;

use crate::{chunks::header::MessageHeader, messages::Message, netconnection::NetConnection};

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

    pub fn netconnection(&self) -> &NetConnection {
        &self.net_connection
    }

    pub async fn route_message<'a>(
        &mut self,
        message: Message<'a>,
        message_stream_id: u32,
        sender: Sender<(MessageHeader, Bytes)>,
    ) {
        // sender.send(message).await;
    }
}
