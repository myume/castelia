use std::collections::HashMap;

use bytes::Bytes;
use thiserror::Error;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info, trace, warn};

use crate::{
    amf::AMF0Value,
    broadcast::{CreateStreamError, MediaType},
    chunks::header::{FullMessageHeader, MessageHeader},
    messages::{
        Message,
        command::{
            CommandMessage,
            command_message_type::{self, COMMAND_AMF0},
        },
    },
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
    #[error("Failed to create stream: {0}")]
    CreateFailure(
        #[source]
        #[from]
        CreateStreamError,
    ),
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

        let mut message = [
            AMF0Value::String("onStatus"),
            AMF0Value::Number(0.0),
            AMF0Value::Null,
            AMF0Value::Object(HashMap::from([
                ("level", AMF0Value::String("status")),
                ("code", AMF0Value::String("NetStream.Publish.Start")),
            ])),
        ];
        match broadcaster.lock().await.create_stream(stream_key).await {
            Ok((stream, stream_id)) => {
                self.stream = Some(stream);
                self.stream_id = Some(stream_id);
                self.state = NetStreamState::Publishing;
                info!("Published stream");
            }
            Err(err) => match err {
                CreateStreamError::AuthError(auth_err) => {
                    warn!("Failed to publish stream: {auth_err}");
                    message[3] = AMF0Value::Object(HashMap::from([
                        ("level", AMF0Value::String("error")),
                        ("code", AMF0Value::String("NetStream.Publish.Denied")),
                    ]));
                }
            },
        };
        let message = message.map(|val| val.serialize()).concat();

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

    async fn handle_play(
        &mut self,
        channel_name: &str,
        broadcaster: Broadcasts,
        send_queue: Sender<SendQueueMessage>,
    ) -> Result<(), HandleError> {
        let Ok(mut receiver) = broadcaster.lock().await.subscribe(channel_name).await else {
            let message = [
                AMF0Value::String("onStatus"),
                AMF0Value::Number(0.0),
                AMF0Value::Null,
                AMF0Value::Object(HashMap::from([
                    ("level", AMF0Value::String("error")),
                    ("code", AMF0Value::String("NetStream.Play.StreamNotFound")),
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
            return Err(HandleError::BroadcastNotFound);
        };

        let message_stream_id = self.id;
        let channel_name = channel_name.to_owned();
        tokio::spawn(async move {
            info!("subscribed to stream {}", channel_name);
            let Ok(metadata) = receiver.receive_metadata().await else {
                return error!("Failed to get metadata for stream");
            };
            if let Err(e) = send_queue
                .send((
                    FullMessageHeader {
                        timestamp: 0,
                        extended_timestamp: None,
                        message_length: metadata.len() as u32,
                        message_type_id: command_message_type::DATA_AMF0,
                        message_stream_id,
                    },
                    metadata,
                ))
                .await
            {
                return error!("Failed to send metadata for stream {e}");
            }

            let Ok(audio_header) = receiver.receive_audio_header().await else {
                return error!("Failed to get audio header for stream");
            };
            if let Err(e) = send_queue
                .send((
                    FullMessageHeader {
                        timestamp: 0,
                        extended_timestamp: None,
                        message_length: audio_header.len() as u32,
                        message_type_id: command_message_type::AUDIO,
                        message_stream_id,
                    },
                    audio_header,
                ))
                .await
            {
                return error!("Failed to send audio_header for stream {e}");
            }

            let Ok(video_header) = receiver.receive_video_header().await else {
                return error!("Failed to get video header for stream");
            };
            if let Err(e) = send_queue
                .send((
                    FullMessageHeader {
                        timestamp: 0,
                        extended_timestamp: None,
                        message_length: video_header.len() as u32,
                        message_type_id: command_message_type::VIDEO,
                        message_stream_id,
                    },
                    video_header,
                ))
                .await
            {
                return error!("Failed to send video header for stream {e}");
            }

            let mut timestamp = 0;
            while let Ok((media_type, header, data)) = receiver.receive_data().await {
                debug!("stream data received");
                timestamp = header.get_timestamp().unwrap_or(timestamp);
                timestamp += header.get_timestamp_delta().unwrap_or(0);
                let message = match media_type {
                    MediaType::Video => CommandMessage::Video(data),
                    MediaType::Audio => CommandMessage::Audio(data),
                };
                let bytes = Message::Command(message).serialize();

                if let Err(e) = send_queue
                    .send((
                        FullMessageHeader {
                            timestamp,
                            extended_timestamp: None,
                            message_length: bytes.len() as u32,
                            message_type_id: match media_type {
                                MediaType::Video => command_message_type::VIDEO,
                                MediaType::Audio => command_message_type::AUDIO,
                            },
                            message_stream_id,
                        },
                        bytes,
                    ))
                    .await
                {
                    return error!("Failed to send message {e}");
                }
            }

            info!("Stream has finished");
        });
        Ok(())
    }

    async fn handle_netstream_command<'a>(
        &mut self,
        command: NetStreamCommand<'a>,
        _transaction_id: f64,
        send_queue: Sender<SendQueueMessage>,
        broadcaster: Broadcasts,
    ) -> Result<(), HandleError> {
        match command {
            NetStreamCommand::Play { stream_name, .. } => {
                self.handle_play(stream_name, broadcaster, send_queue)
                    .await?
            }
            NetStreamCommand::Play2 { .. } => {
                // only allow play1 for now
                return Err(HandleError::UnsupportedCommand("play2".into()));
            }
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
            // always receive video and audio for now
            NetStreamCommand::ReceiveAudio { .. } => {}
            NetStreamCommand::ReceiveVideo { .. } => {}
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
            NetStreamCommand::Pause { .. } => {
                // supports only live streaming so don't handle this for now
                return Err(HandleError::UnsupportedCommand("pause".into()));
            }
        }

        Ok(())
    }

    async fn handle_metadata<'a>(
        &self,
        metadata: Vec<AMF0Value<'a>>,
        broadcaster: Broadcasts,
    ) -> Result<(), HandleError> {
        let Some(stream_key) = &self.stream_id else {
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

    async fn handle_media_header(
        &mut self,
        header: Bytes,
        media_type: MediaType,
        broadcaster: Broadcasts,
    ) -> Result<(), HandleError> {
        let Some(stream_key) = &self.stream_id else {
            return Err(HandleError::BroadcastNotFound);
        };

        match media_type {
            MediaType::Video => {
                broadcaster
                    .lock()
                    .await
                    .set_stream_video_header(stream_key, header)
                    .await
                    .map_err(|_| {
                        error!("failed to set video header for stream");
                        HandleError::BroadcastNotFound
                    })?;
                self.video_header_sent = true;
            }
            MediaType::Audio => {
                broadcaster
                    .lock()
                    .await
                    .set_stream_audio_header(stream_key, header)
                    .await
                    .map_err(|_| {
                        error!("failed to set audio header for stream");
                        HandleError::BroadcastNotFound
                    })?;
                self.audio_header_sent = true;
            }
        }
        info!("{media_type:?} header sent");

        Ok(())
    }

    async fn handle_media(
        &mut self,
        bytes: Bytes,
        media_type: MediaType,
        message_header: MessageHeader,
    ) -> Result<(), HandleError> {
        trace!("sending {media_type:?} data");
        if let Some(stream) = &mut self.stream {
            stream
                .send_data(bytes, media_type, message_header)
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
        message_header: MessageHeader,
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
                if !self.audio_header_sent {
                    self.handle_media_header(bytes, MediaType::Audio, broadcaster)
                        .await
                } else {
                    self.handle_media(bytes, MediaType::Audio, message_header)
                        .await
                }
            }

            CommandMessage::Video(bytes) => {
                trace!("sending video data");
                if !self.video_header_sent {
                    self.handle_media_header(bytes, MediaType::Video, broadcaster)
                        .await
                } else {
                    self.handle_media(bytes, MediaType::Video, message_header)
                        .await
                }
            }
        }
    }
}
