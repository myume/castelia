use tokio::sync::mpsc::Sender;

use crate::{
    messages::{protocol_control::ProtocolControlMessage, user_control::UserControlMessage},
    netconnection::{NetConnection, command::NetConnectionCommand},
    rtmp::SendQueueMessage,
};

impl NetConnection {
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

    pub async fn handle_command<'a>(
        &mut self,
        command: NetConnectionCommand<'a>,
        send_queue: Sender<SendQueueMessage>,
    ) {
    }
}
