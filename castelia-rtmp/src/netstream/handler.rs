use thiserror::Error;
use tokio::sync::mpsc::Sender;

use crate::{
    messages::command::CommandMessage,
    netstream::{NetStream, NetStreamState, command::NetStreamCommand},
    rtmp::SendQueueMessage,
};

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("Unsupported command \"{0}\"")]
    UnsupportedCommand(String),
}

impl NetStream {
    async fn handle_publish(
        &mut self,
        publishing_name: &str,
        publishing_type: &str,
        send_queue: Sender<SendQueueMessage>,
    ) -> Result<(), HandleError> {
        // assume no authentication for now, we will need to add it later

        self.state = NetStreamState::Publishing;
        Ok(())
    }

    async fn handle_netstream_command<'a>(
        &mut self,
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
            NetStreamCommand::DeleteStream { .. } => {
                // delete stream command should be handled by message router.
                // the alternate approach is to mark the stream with a tombstone and clean up all
                // streams with tombstones. But that requires scanning all streams, which seems
                // less performant, though it probably doesn't matter because there usually aren't many
                // message streams active at a time.
                return Err(HandleError::UnsupportedCommand("deleteStream".to_owned()));
            }
            NetStreamCommand::CloseStream { .. } => {
                self.state = NetStreamState::Closed;
            }
            NetStreamCommand::ReceiveAudio { should_receive } => todo!(),
            NetStreamCommand::ReceiveVideo { should_receive } => todo!(),
            NetStreamCommand::Publish {
                publishing_name,
                publishing_type,
            } => {
                self.handle_publish(publishing_name, publishing_type, send_queue)
                    .await?
            }
            NetStreamCommand::Seek { .. } => {
                return Err(HandleError::UnsupportedCommand("seek".to_owned()));
            }
            NetStreamCommand::Pause {
                is_paused,
                milliseconds,
            } => todo!(),
        }

        Ok(())
    }

    pub async fn handle_message<'a>(
        &mut self,
        message: CommandMessage<'a>,
        send_queue: Sender<SendQueueMessage>,
    ) -> Result<(), HandleError> {
        match message {
            CommandMessage::NetConnection(_) => {
                Err(HandleError::UnsupportedCommand("NetConnection".to_owned()))
            }
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
