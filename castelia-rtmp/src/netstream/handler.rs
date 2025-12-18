use std::collections::HashMap;

use bytes::Bytes;
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tracing::{error, trace};

use crate::{
    amf::AMF0Value,
    chunks::header::FullMessageHeader,
    messages::command::{CommandMessage, command_message_type::COMMAND_AMF0},
    netstream::{NetStream, NetStreamState, command::NetStreamCommand},
    rtmp::{Broadcasts, SendQueueMessage},
};

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("Unsupported command \"{0}\"")]
    UnsupportedCommand(String),
    #[error("Only live broadcasts are supported")]
    NoneLiveBroadcast,
}

impl NetStream {
    async fn handle_publish(
        &mut self,
        stream_key: &str,
        publishing_type: &str,
        send_queue: Sender<SendQueueMessage>,
        broadcaster: Broadcasts,
    ) -> Result<(), HandleError> {
        if publishing_type != "live" {
            return Err(HandleError::NoneLiveBroadcast);
        }

        self.stream = Some(broadcaster.lock().await.create_stream(stream_key).await);
        self.state = NetStreamState::Publishing;
        trace!("Stream created");

        let message = [
            AMF0Value::String("onStatus"),
            AMF0Value::Number(0.0),
            AMF0Value::Null,
            AMF0Value::Object(HashMap::from([
                ("level", AMF0Value::String("status")),
                ("code", AMF0Value::String("NetStream.Publish.Start")),
            ])),
        ]
        .map(|val| val.serialize())
        .concat();
        if let Err(e) = send_queue
            .send((
                FullMessageHeader {
                    timestamp: 0,
                    extended_timestamp: None,
                    message_length: message.len() as u32,
                    message_type_id: COMMAND_AMF0,
                    message_stream_id: self.id,
                },
                Bytes::from(message),
            ))
            .await
        {
            error!("Failed to send message: {e}");
        };
        Ok(())
    }

    async fn handle_netstream_command<'a>(
        &mut self,
        command: NetStreamCommand<'a>,
        transaction_id: f64,
        send_queue: Sender<SendQueueMessage>,
        broadcaster: Broadcasts,
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
                self.handle_publish(publishing_name, publishing_type, send_queue, broadcaster)
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
        broadcaster: Broadcasts,
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
                self.handle_netstream_command(command, transaction_id, send_queue, broadcaster)
                    .await
            }
            CommandMessage::Data(amf0_values) => todo!(),
            CommandMessage::Audio(bytes) => todo!(),
            CommandMessage::Video(bytes) => todo!(),
        }
    }
}
