use thiserror::Error;

use crate::messages::{
    command::{CommandMessage, command_message_type},
    protocol_control::{ProtolControlMessage, protocol_control_type},
    user_control::{USER_CONTROL_TYPE, UserControlMessage},
};

pub mod command;
pub mod protocol_control;
pub mod user_control;

#[derive(Error, Debug)]
pub enum ParseMessageError {
    #[error("Invalid message type id: {0}")]
    InvalidMessageTypeId(u8),

    #[error("Invalid message size")]
    InvalidMessageSize,
}

#[derive(Debug)]
pub enum Message {
    Protocol(ProtolControlMessage),
    UserControl(UserControlMessage),
    Command(CommandMessage),
}

impl Message {
    pub fn parse_message(buf: &[u8], message_type_id: u8) -> Result<Self, ParseMessageError> {
        Ok(match message_type_id {
            protocol_control_type::SET_CHUNK_SIZE
            | protocol_control_type::ABORT
            | protocol_control_type::ACK
            | protocol_control_type::WINDOW_ACK_SIZE
            | protocol_control_type::SET_PEER_BANDWIDTH => {
                Self::Protocol(ProtolControlMessage::parse_message(buf, &message_type_id)?)
            }

            USER_CONTROL_TYPE => {
                Self::UserControl(UserControlMessage::parse_message(buf, &message_type_id)?)
            }

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
}
