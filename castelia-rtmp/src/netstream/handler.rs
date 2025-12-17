use thiserror::Error;
use tokio::sync::mpsc::Sender;

use crate::{
    messages::command::CommandMessage,
    netstream::{NetStream, command::NetStreamCommand},
    rtmp::SendQueueMessage,
};

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("Found netconnection command for netstream")]
    NetconnectionCommand,
}

impl NetStream {
    async fn handle_publish(
        &self,
        publishing_name: &str,
        publishing_type: &str,
        send_queue: Sender<SendQueueMessage>,
    ) -> Result<(), HandleError> {
        todo!()
    }

    async fn handle_netstream_command<'a>(
        &self,
        command: NetStreamCommand<'a>,
        transaction_id: f64,
        send_queue: Sender<SendQueueMessage>,
    ) -> Result<(), HandleError> {
        match command {
            NetStreamCommand::Play {
                stream_name,
                start,
                duration,
                reset,
            } => todo!(),
            NetStreamCommand::Play2 { parameters } => todo!(),
            NetStreamCommand::DeleteStream { stream_id } => todo!(),
            NetStreamCommand::CloseStream { stream_id } => todo!(),
            NetStreamCommand::ReceiveAudio { should_receive } => todo!(),
            NetStreamCommand::ReceiveVideo { should_receive } => todo!(),
            NetStreamCommand::Publish {
                publishing_name,
                publishing_type,
            } => {
                self.handle_publish(publishing_name, publishing_type, send_queue)
                    .await
            }
            NetStreamCommand::Seek { milliseconds } => todo!(),
            NetStreamCommand::Pause {
                is_paused,
                milliseconds,
            } => todo!(),
        }
    }

    pub async fn handle_message<'a>(
        &self,
        message: CommandMessage<'a>,
        send_queue: Sender<SendQueueMessage>,
    ) -> Result<(), HandleError> {
        match message {
            CommandMessage::NetConnection(_) => Err(HandleError::NetconnectionCommand),
            CommandMessage::NetStreamCommand {
                command,
                transaction_id,
                ..
            } => {
                self.handle_netstream_command(command, transaction_id, send_queue)
                    .await
            }
            CommandMessage::Data(amf0_values) => todo!(),
            CommandMessage::Audio(bytes) => todo!(),
            CommandMessage::Video(bytes) => todo!(),
        }
    }
}
