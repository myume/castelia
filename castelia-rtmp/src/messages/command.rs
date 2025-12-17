use bytes::{Bytes, BytesMut};
use thiserror::Error;
use tracing::warn;

use crate::{
    amf::{self, AMF0Value},
    netconnection::command::NetConnectionCommand,
    netstream::command::NetStreamCommand,
};

pub mod command_message_type {
    pub const COMMAND_AMF0: u8 = 20;
    pub const COMMAND_AMF3: u8 = 17;
    pub const DATA_AMF0: u8 = 18;
    pub const DATA_AMF3: u8 = 15;
    pub const SHARED_OBJECT_AMF0: u8 = 16;
    pub const SHARED_OBJECT_AMF3: u8 = 14;
    pub const AUDIO: u8 = 8;
    pub const VIDEO: u8 = 9;
    pub const AGGREGATE: u8 = 22;
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid message type: {0}")]
    InvalidMessageType(u8),
    #[error("AMF3 encoding is unsupported")]
    UnsupportedEncoding,
    #[error("Failed to decode message: {0}")]
    DecodeError(
        #[source]
        #[from]
        amf::DecodeError,
    ),
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Failed to cast AMF encoded value: {0}")]
    CastError(
        #[source]
        #[from]
        amf::CastError,
    ),
}

#[derive(Debug, PartialEq)]
pub enum CommandMessage<'a> {
    NetConnection(NetConnectionCommand<'a>),
    NetStreamCommand {
        command: NetStreamCommand<'a>,
        transaction_id: f64,
        command_object: amf::AMF0Value<'a>,
    },
    Data(Vec<amf::AMF0Value<'a>>),
    Audio(Bytes),
    Video(Bytes),
    // leave unsupported for now, unless we see it in use
    // SharedObject,
    // Aggregate,
}

impl<'a> CommandMessage<'a> {
    pub fn parse_message(
        bytes: &'a Bytes,
        message_type_id: &u8,
    ) -> Result<CommandMessage<'a>, ParseError> {
        match *message_type_id {
            command_message_type::COMMAND_AMF0 => CommandMessage::parse_command(bytes),
            command_message_type::DATA_AMF0 => CommandMessage::parse_data_message(bytes),
            command_message_type::AUDIO => Ok(CommandMessage::Audio(bytes.clone())),
            command_message_type::VIDEO => Ok(CommandMessage::Video(bytes.clone())),

            command_message_type::SHARED_OBJECT_AMF0 => {
                warn!("Unhandled shared object message found");
                Err(ParseError::InvalidMessageType(*message_type_id))
            }
            command_message_type::AGGREGATE => {
                warn!("Unhandled aggregate message found");
                Err(ParseError::InvalidMessageType(*message_type_id))
            }
            command_message_type::COMMAND_AMF3
            | command_message_type::DATA_AMF3
            | command_message_type::SHARED_OBJECT_AMF3 => {
                warn!("Unsupported AMF3 encoded message found");
                Err(ParseError::UnsupportedEncoding)
            }
            e => Err(ParseError::InvalidMessageType(e)),
        }
    }

    fn parse_data_message(buf: &'a [u8]) -> Result<CommandMessage<'a>, ParseError> {
        let mut data = Vec::new();
        let mut decoder = amf::Decoder::new(buf);
        while !decoder.get_buf()?.is_empty() {
            data.push(decoder.decode()?);
        }

        Ok(CommandMessage::Data(data))
    }

    fn parse_command(buf: &'a [u8]) -> Result<CommandMessage<'a>, ParseError> {
        CommandMessage::parse_netstream_command(buf)
            .or(CommandMessage::parse_netconnection_command(buf))
    }

    fn parse_netstream_command(buf: &'a [u8]) -> Result<CommandMessage<'a>, ParseError> {
        let mut decoder = amf::Decoder::new(buf);
        let (command_type, transaction_id, command_object) =
            CommandMessage::parse_base_command(&mut decoder)?;
        let command = NetStreamCommand::parse(command_type, decoder.get_buf()?)?;

        Ok(CommandMessage::NetStreamCommand {
            command,
            transaction_id,
            command_object,
        })
    }

    fn parse_base_command<'d>(
        decoder: &'d mut amf::Decoder<'a>,
    ) -> Result<(&'a str, f64, amf::AMF0Value<'a>), ParseError> {
        let command = decoder.decode()?.try_into()?;
        let transaction_id = decoder.decode()?.try_into()?;
        let command_object = decoder.decode()?;
        Ok((command, transaction_id, command_object))
    }

    fn parse_netconnection_command(buf: &'a [u8]) -> Result<CommandMessage<'a>, ParseError> {
        let mut decoder = amf::Decoder::new(buf);
        let (command_type, transaction_id, command_object) =
            CommandMessage::parse_base_command(&mut decoder)?;
        Ok(CommandMessage::NetConnection(NetConnectionCommand {
            command_type: command_type.into(),
            transaction_id,
            command_object,
        }))
    }

    pub fn serialize(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        match self {
            CommandMessage::NetConnection(net_connection_command) => {
                bytes.extend_from_slice(
                    &AMF0Value::String(&net_connection_command.command_type.to_string())
                        .serialize(),
                );
                bytes.extend_from_slice(
                    &AMF0Value::Number(net_connection_command.transaction_id).serialize(),
                );
                bytes.extend_from_slice(&net_connection_command.command_object.serialize());
            }
            CommandMessage::NetStreamCommand {
                command,
                transaction_id,
                command_object,
            } => {
                bytes.extend_from_slice(&AMF0Value::String(command.name()).serialize());
                bytes.extend_from_slice(&AMF0Value::Number(*transaction_id).serialize());
                bytes.extend_from_slice(&command_object.serialize());

                match command {
                    NetStreamCommand::Play {
                        stream_name,
                        start,
                        duration,
                        reset,
                    } => {
                        bytes.extend_from_slice(&AMF0Value::String(stream_name).serialize());
                        if let Some(start) = start {
                            bytes.extend_from_slice(&AMF0Value::Number(*start).serialize());
                        }
                        if let Some(duration) = duration {
                            bytes.extend_from_slice(&AMF0Value::Number(*duration).serialize());
                        }
                        if let Some(reset) = reset {
                            bytes.extend_from_slice(&AMF0Value::Boolean(*reset).serialize());
                        }
                    }
                    NetStreamCommand::Play2 { parameters } => {
                        bytes.extend_from_slice(&parameters.serialize());
                    }
                    NetStreamCommand::DeleteStream { stream_id } => {
                        bytes.extend_from_slice(&AMF0Value::Number(*stream_id as f64).serialize());
                    }
                    NetStreamCommand::CloseStream { stream_id } => {
                        bytes.extend_from_slice(&AMF0Value::Number(*stream_id as f64).serialize());
                    }
                    NetStreamCommand::ReceiveAudio { should_receive } => {
                        bytes.extend_from_slice(&AMF0Value::Boolean(*should_receive).serialize());
                    }
                    NetStreamCommand::ReceiveVideo { should_receive } => {
                        bytes.extend_from_slice(&AMF0Value::Boolean(*should_receive).serialize());
                    }
                    NetStreamCommand::Publish {
                        publishing_name,
                        publishing_type,
                    } => {
                        bytes.extend_from_slice(&AMF0Value::String(publishing_name).serialize());
                        bytes.extend_from_slice(&AMF0Value::String(publishing_type).serialize());
                    }
                    NetStreamCommand::Seek { milliseconds } => {
                        bytes.extend_from_slice(&AMF0Value::Number(*milliseconds).serialize());
                    }
                    NetStreamCommand::Pause {
                        is_paused,
                        milliseconds,
                    } => {
                        bytes.extend_from_slice(&AMF0Value::Boolean(*is_paused).serialize());
                        bytes.extend_from_slice(&AMF0Value::Number(*milliseconds).serialize());
                    }
                }
            }
            CommandMessage::Data(amf0_values) => {
                for value in amf0_values {
                    bytes.extend_from_slice(&value.serialize());
                }
            }
            CommandMessage::Audio(bytes) => return bytes.clone(),
            CommandMessage::Video(bytes) => return bytes.clone(),
        }
        bytes.into()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::netconnection::command::NetConnectionCommandType;

    use super::*;

    #[test]
    fn test_serialize_parse_netconnection_command() {
        let expected = CommandMessage::NetConnection(NetConnectionCommand {
            command_type: NetConnectionCommandType::Connect,
            transaction_id: rand::random(),
            command_object: AMF0Value::Object(HashMap::from([("app", AMF0Value::String("live"))])),
        });

        let bytes = expected.serialize();
        let actual = CommandMessage::parse_command(&bytes).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_parse_play_command() {
        let expected = CommandMessage::NetStreamCommand {
            command: NetStreamCommand::Play {
                stream_name: "test",
                start: Some(rand::random()),
                duration: Some(rand::random()),
                reset: Some(rand::random()),
            },
            transaction_id: rand::random(),
            command_object: AMF0Value::Null,
        };
        let bytes = expected.serialize();
        let actual = CommandMessage::parse_command(&bytes).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_parse_play_command_no_body() {
        let expected = CommandMessage::NetStreamCommand {
            command: NetStreamCommand::Play {
                stream_name: "test",
                start: None,
                duration: None,
                reset: None,
            },
            transaction_id: 0.0,
            command_object: AMF0Value::Null,
        };
        let bytes = expected.serialize();

        let expected_bytes = [
            AMF0Value::String("play"),
            AMF0Value::Number(0.0),
            AMF0Value::Null,
            AMF0Value::String("test"),
        ]
        .map(|val| val.serialize())
        .concat();
        assert_eq!(bytes, Bytes::from(expected_bytes));
        let actual = CommandMessage::parse_command(&bytes).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_play_command_from_amf() {
        let expected = CommandMessage::NetStreamCommand {
            command: NetStreamCommand::Play {
                stream_name: "test",
                start: None,
                duration: None,
                reset: None,
            },
            transaction_id: 0.0,
            command_object: AMF0Value::Null,
        };
        let bytes = [
            AMF0Value::String("play"),
            AMF0Value::Number(0.0),
            AMF0Value::Null,
            AMF0Value::String("test"),
        ]
        .map(|val| val.serialize())
        .concat();
        let actual = CommandMessage::parse_command(&bytes).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_parse_play2_command() {
        let expected = CommandMessage::NetStreamCommand {
            command: NetStreamCommand::Play2 {
                parameters: AMF0Value::Object(HashMap::from([("test", AMF0Value::String("test"))])),
            },
            transaction_id: rand::random(),
            command_object: AMF0Value::Null,
        };
        let bytes = expected.serialize();
        let actual = CommandMessage::parse_netstream_command(&bytes).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_parse_delete_stream_command() {
        let expected = CommandMessage::NetStreamCommand {
            command: NetStreamCommand::DeleteStream {
                stream_id: rand::random(),
            },
            transaction_id: 0.0,
            command_object: AMF0Value::Null,
        };
        let bytes = expected.serialize();
        let actual = CommandMessage::parse_command(&bytes).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_parse_receive_audio_command() {
        let expected = CommandMessage::NetStreamCommand {
            command: NetStreamCommand::ReceiveAudio {
                should_receive: rand::random(),
            },
            transaction_id: rand::random(),
            command_object: AMF0Value::Null,
        };
        let bytes = expected.serialize();
        let actual = CommandMessage::parse_command(&bytes).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_parse_receive_video_command() {
        let expected = CommandMessage::NetStreamCommand {
            command: NetStreamCommand::ReceiveVideo {
                should_receive: rand::random(),
            },
            transaction_id: rand::random(),
            command_object: AMF0Value::Null,
        };
        let bytes = expected.serialize();
        let actual = CommandMessage::parse_command(&bytes).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_parse_publish_command() {
        let expected = CommandMessage::NetStreamCommand {
            command: NetStreamCommand::Publish {
                publishing_name: "stream_key",
                publishing_type: "live",
            },
            transaction_id: rand::random(),
            command_object: AMF0Value::Null,
        };
        let bytes = expected.serialize();
        let actual = CommandMessage::parse_command(&bytes).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_parse_seek_command() {
        let expected = CommandMessage::NetStreamCommand {
            command: NetStreamCommand::Seek {
                milliseconds: rand::random(),
            },
            transaction_id: rand::random(),
            command_object: AMF0Value::Null,
        };
        let bytes = expected.serialize();
        let actual = CommandMessage::parse_command(&bytes).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_serialize_parse_pause_command() {
        let expected = CommandMessage::NetStreamCommand {
            command: NetStreamCommand::Pause {
                is_paused: rand::random(),
                milliseconds: rand::random(),
            },
            transaction_id: 0.0,
            command_object: AMF0Value::Null,
        };
        let bytes = expected.serialize();
        let actual = CommandMessage::parse_command(&bytes).unwrap();
        assert_eq!(actual, expected);
    }
}
