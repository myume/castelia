use thiserror::Error;

use crate::{
    amf::{self},
    netconnection::NetConnectionCommandType,
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
pub enum CommandMessage<'a> {
    NetConnectionCommand {
        command_type: NetConnectionCommandType<'a>,
        transaction_id: f64,
        command_object: amf::AMF0Value<'a>,
    },
    NetStreamCommand {},
    Data,
    SharedObject,
    Audio(&'a [u8]),
    Video(&'a [u8]),
    Aggregate,
}

impl<'a> CommandMessage<'a> {
    pub fn parse_message(
        buf: &'a [u8],
        message_type_id: &u8,
    ) -> Result<CommandMessage<'a>, ParseError> {
        match *message_type_id {
            command_message_type::COMMAND_AMF0 => CommandMessage::parse_command(buf),
            // command_message_type::DATA_AMF0 => CommandMessage::Data,
            // command_message_type::SHARED_OBJECT_AMF0 => CommandMessage::SharedObject,
            command_message_type::AUDIO => Ok(CommandMessage::Audio(buf)),
            command_message_type::VIDEO => Ok(CommandMessage::Video(buf)),
            // command_message_type::AGGREGATE => {}
            command_message_type::COMMAND_AMF3
            | command_message_type::DATA_AMF3
            | command_message_type::SHARED_OBJECT_AMF3 => Err(ParseError::UnsupportedEncoding),
            e => Err(ParseError::InvalidMessageType(e)),
        }
    }

    fn parse_command(buf: &'a [u8]) -> Result<CommandMessage<'a>, ParseError> {
        let mut decoder = amf::Decoder::new(buf);
        let command = match decoder.decode()? {
            amf::AMF0Value::String(command) => command,
            val => {
                return Err(ParseError::InvalidCommand(format!(
                    "Expected string for command, found: {:?}",
                    val
                )));
            }
        };
        let transaction_id = match decoder.decode()? {
            amf::AMF0Value::Number(num) => num,
            val => {
                return Err(ParseError::InvalidTransationId(format!(
                    "Expected number for transaction id, found: {:?}",
                    val
                )));
            }
        };
        let command_object = decoder.decode()?;

        Ok(CommandMessage::NetConnectionCommand {
            command_type: command.into(),
            transaction_id,
            command_object,
        })
    }
}
