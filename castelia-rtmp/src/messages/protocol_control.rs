use thiserror::Error;

use crate::messages::ParseMessageError;

pub mod protocol_control_type {
    pub const SET_CHUNK_SIZE: u8 = 1;
    pub const ABORT: u8 = 2;
    pub const ACK: u8 = 3;
    pub const WINDOW_ACK_SIZE: u8 = 5;
    pub const SET_PEER_BANDWIDTH: u8 = 6;
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid message size")]
    InvalidMessageSize,
    #[error("Invalid message type id: {0}")]
    InvalidMessageTypeId(u8),
}

impl From<ParseError> for ParseMessageError {
    fn from(value: ParseError) -> Self {
        match value {
            ParseError::InvalidMessageSize => Self::InvalidMessageSize,
            ParseError::InvalidMessageTypeId(id) => Self::InvalidMessageTypeId(id),
        }
    }
}

#[derive(Debug)]
pub enum ProtolControlMessage {
    SetChunkSize(u32),
    Abort(u32),
    Ack(u32),
    AckWindowSize(u32),
    SetPeerBandwidth { limit_type: u8, window_size: u32 },
}

impl ProtolControlMessage {
    pub fn parse_message(buf: &[u8], message_type_id: &u8) -> Result<Self, ParseError> {
        let data = u32::from_be_bytes(
            buf.get(..4)
                .ok_or(ParseError::InvalidMessageSize)?
                .try_into()
                .map_err(|_| ParseError::InvalidMessageSize)?,
        );
        Ok(match *message_type_id {
            protocol_control_type::SET_CHUNK_SIZE => Self::SetChunkSize(data),
            protocol_control_type::ABORT => Self::Abort(data),
            protocol_control_type::ACK => Self::Ack(data),
            protocol_control_type::WINDOW_ACK_SIZE => Self::AckWindowSize(data),
            protocol_control_type::SET_PEER_BANDWIDTH => Self::SetPeerBandwidth {
                window_size: data,
                limit_type: *buf.get(5).ok_or(ParseError::InvalidMessageSize)?,
            },
            _ => return Err(ParseError::InvalidMessageTypeId(*message_type_id)),
        })
    }
}
