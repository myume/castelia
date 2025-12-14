use tokio::sync::mpsc::Sender;

use crate::{
    messages::{protocol_control::ProtocolControlMessage, user_control::UserControlMessage},
    rtmp::SendQueueMessage,
};

pub mod command;

#[derive(Debug)]
pub struct NetConnection {
    max_chunk_size: u32,
}

impl NetConnection {
    pub fn new() -> Self {
        NetConnection {
            max_chunk_size: 4096,
        }
    }

    pub fn max_chunk_size(&self) -> u32 {
        self.max_chunk_size
    }

    pub async fn handle_protocol_message(
        &mut self,
        message: ProtocolControlMessage,
        send_queue: Sender<SendQueueMessage>,
    ) {
    }

    pub async fn handle_user_control_message(
        &mut self,
        message: UserControlMessage,
        send_queue: Sender<SendQueueMessage>,
    ) {
    }
}
