use thiserror::Error;

use crate::messages::{
    command::CommandMessage, protocol_control::ProtolControlMessage,
    user_control::UserControlMessage,
};

pub mod command;
pub mod protocol_control;
pub mod user_control;

#[derive(Error, Debug)]
pub enum ParseMessageError {
    #[error("Invalid message type id: {0}")]
    InvalidMessageTypeId(u8),
}

#[derive(Debug)]
pub enum Message {
    Protocol(ProtolControlMessage),
    UserControl(UserControlMessage),
    Command(CommandMessage),
}

impl Message {
    pub fn parse_message(buf: &[u8], message_type_id: u8) -> Result<Self, ParseMessageError> {
        match message_type_id {
            1 | 2 | 3 | 5 | 6 => Self::Protocol(ProtolControlMessage::parse_message(buf))?,
            4 => Self::UserControl(UserControlMessage::parse_message(buf))?,
            20 | 17 | 18 | 15 | 19 | 16 | 8 | 9 | 22 => {
                Self::Command(CommandMessage::parse_message(buf))?
            }
            id => Err(ParseMessageError::InvalidMessageTypeId(id)),
        }
    }
}
