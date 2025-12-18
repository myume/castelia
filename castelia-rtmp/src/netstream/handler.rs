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
    #[error("Broadcast could not be found")]
    BroadcastNotFound,
    #[error("Failed to send message")]
    SendError(String),
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
        self.stream_key = Some(stream_key.to_owned());
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

    async fn handle_metadata<'a>(
        &self,
        metadata: Vec<AMF0Value<'a>>,
        broadcaster: Broadcasts,
    ) -> Result<(), HandleError> {
        let Some(stream_key) = &self.stream_key else {
            return Err(HandleError::BroadcastNotFound);
        };
        if broadcaster
            .lock()
            .await
            .set_stream_metadata(
                stream_key,
                metadata.iter().flat_map(|val| val.serialize()).collect(),
            )
            .await
            .is_err()
        {
            error!("failed to set metadata for stream.");
            return Err(HandleError::BroadcastNotFound);
        }
        Ok(())
    }

    async fn handle_media(&mut self, bytes: Bytes) -> Result<(), HandleError> {
        if let Some(stream) = &mut self.stream {
            stream
                .send_data(bytes)
                .await
                .map_err(|e| HandleError::SendError(e.to_string()))
        } else {
            Err(HandleError::BroadcastNotFound)
        }
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
            CommandMessage::Data(amf0_values) => {
                trace!("setting stream metadata");
                self.handle_metadata(amf0_values, broadcaster).await
            }
            CommandMessage::Audio(bytes) => {
                trace!("sending audio data");
                self.handle_media(bytes).await
            }

            CommandMessage::Video(bytes) => {
                trace!("sending video data");
                self.handle_media(bytes).await
            }
        }
    }
}
