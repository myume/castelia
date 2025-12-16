use tokio::sync::mpsc::Sender;

use crate::{messages::command::CommandMessage, netstream::NetStream, rtmp::SendQueueMessage};

impl NetStream {
    pub fn handle_message(&self, message: CommandMessage, send_queue: Sender<SendQueueMessage>) {}
}
