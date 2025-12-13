use crate::{messages::Message, netconnection::NetConnection};

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

    pub fn route_message(&mut self, message: Message, message_stream_id: u32) {}
}
