use bytes::Bytes;
use thiserror::Error;

use crate::messages::{
    command::{CommandMessage, command_message_type},
    protocol_control::{ProtocolControlMessage, protocol_control_type},
    user_control::{USER_CONTROL_TYPE, UserControlMessage},
};

pub mod command;
pub mod protocol_control;
pub(crate) mod router;
pub mod user_control;

#[derive(Error, Debug)]
pub enum ParseMessageError {
    #[error("Invalid message type id: {0}")]
    InvalidMessageTypeId(u8),
    #[error("Invalid message size")]
    InvalidMessageSize,
    #[error("Invalid command")]
    BadCommandMessage(
        #[source]
        #[from]
        command::ParseError,
    ),
    #[error("Invalid user control message")]
    BadUserControl(
        #[source]
        #[from]
        user_control::ParseError,
    ),
}

#[derive(Debug)]
pub enum Message<'a> {
    Protocol(ProtocolControlMessage),
    UserControl(UserControlMessage),
    Command(CommandMessage<'a>),
}

impl<'a> Message<'a> {
    pub fn get_type_id(&self) -> u8 {
        match self {
            Message::Protocol(protocol_control_message) => match protocol_control_message {
                ProtocolControlMessage::SetChunkSize(_) => protocol_control_type::SET_CHUNK_SIZE,
                ProtocolControlMessage::Abort(_) => protocol_control_type::ABORT,
                ProtocolControlMessage::Ack(_) => protocol_control_type::ACK,
                ProtocolControlMessage::AckWindowSize(_) => protocol_control_type::WINDOW_ACK_SIZE,
                ProtocolControlMessage::SetPeerBandwidth(_) => {
                    protocol_control_type::SET_PEER_BANDWIDTH
                }
            },
            Message::UserControl(_) => USER_CONTROL_TYPE,
            Message::Command(command_message) => match command_message {
                CommandMessage::NetConnection(_) => command_message_type::COMMAND_AMF0,
                CommandMessage::NetStreamCommand { .. } => command_message_type::COMMAND_AMF0,
                CommandMessage::Data(_) => command_message_type::DATA_AMF0,
                CommandMessage::Audio(_) => command_message_type::AUDIO,
                CommandMessage::Video(_) => command_message_type::VIDEO,
            },
        }
    }

    pub fn parse_message(buf: &'a Bytes, message_type_id: u8) -> Result<Self, ParseMessageError> {
        Ok(match message_type_id {
            protocol_control_type::SET_CHUNK_SIZE
            | protocol_control_type::ABORT
            | protocol_control_type::ACK
            | protocol_control_type::WINDOW_ACK_SIZE
            | protocol_control_type::SET_PEER_BANDWIDTH => Self::Protocol(
                ProtocolControlMessage::parse_message(buf, &message_type_id)?,
            ),

            USER_CONTROL_TYPE => Self::UserControl(UserControlMessage::parse_message(buf)?),

            command_message_type::COMMAND_AMF0
            | command_message_type::COMMAND_AMF3
            | command_message_type::DATA_AMF0
            | command_message_type::DATA_AMF3
            | command_message_type::SHARED_OBJECT_AMF0
            | command_message_type::SHARED_OBJECT_AMF3
            | command_message_type::AUDIO
            | command_message_type::VIDEO
            | command_message_type::AGGREGATE => {
                Self::Command(CommandMessage::parse_message(buf, &message_type_id)?)
            }
            id => return Err(ParseMessageError::InvalidMessageTypeId(id)),
        })
    }

    pub fn serialize(&self) -> Bytes {
        match self {
            Message::Protocol(protocol_control_message) => protocol_control_message.serialize(),
            Message::UserControl(user_control_message) => user_control_message.serialize(),
            Message::Command(command_message) => command_message.serialize(),
        }
    }
}
