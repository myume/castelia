use thiserror::Error;

use crate::{
    amf::{self},
    messages::ParseMessageError,
    netconnection::NetConnectionCommand,
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
    #[error("Invalid transaction id: {0}")]
    InvalidTransationId(String),
}

#[derive(Debug)]
pub enum Command<'a> {
    NetConnectionCommand(NetConnectionCommand<'a>),
}

#[derive(Debug)]
pub enum CommandMessage<'a> {
    Command(Command<'a>),
    Data,
    SharedObject,
    Audio,
    Video,
    Aggregate,
}

impl<'a> CommandMessage<'a> {
    pub fn parse_message(
        buf: &'a [u8],
        message_type_id: &u8,
    ) -> Result<CommandMessage<'a>, ParseError> {
        Ok(match *message_type_id {
            command_message_type::COMMAND_AMF0 => CommandMessage::parse_command(buf),
            // command_message_type::DATA_AMF0 => CommandMessage::Data,
            // command_message_type::SHARED_OBJECT_AMF0 => CommandMessage::SharedObject,
            // command_message_type::AUDIO => {}
            // command_message_type::VIDEO => {}
            // command_message_type::AGGREGATE => {}
            command_message_type::COMMAND_AMF3
            | command_message_type::DATA_AMF3
            | command_message_type::SHARED_OBJECT_AMF3 => Err(ParseError::UnsupportedEncoding),
            e => Err(ParseError::InvalidMessageType(e)),
        }?)
    }

    fn parse_command(buf: &'a [u8]) -> Result<CommandMessage<'a>, ParseError> {
        let mut decoder = amf::Decoder::new(buf);
        let command = match decoder.decode()? {
            amf::AMF0Value::String(command) => command,
            e => {
                return Err(ParseError::InvalidCommand(format!(
                    "Expected string for command, found: {:?}",
                    e
                )));
            }
        };
        let transaction_id = decoder.decode()?;
        let command_object = decoder.decode()?;

        Ok(CommandMessage::Command(Command::NetConnectionCommand(
            NetConnectionCommand {
                command_type: command.try_into().map_err(ParseError::InvalidCommand)?,
                transaction_id: 1.0,
                command_object,
            },
        )))
    }
}
