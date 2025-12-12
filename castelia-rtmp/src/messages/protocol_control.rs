use std::array::TryFromSliceError;

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
    InvalidMessageSize(
        #[source]
        #[from]
        TryFromSliceError,
    ),
}

impl From<ParseError> for ParseMessageError {
    fn from(value: ParseError) -> Self {
        match value {
            ParseError::InvalidMessageSize(_) => Self::InvalidMessageSize,
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
        Ok(match *message_type_id {
            protocol_control_type::SET_CHUNK_SIZE => {
                Self::SetChunkSize(u32::from_be_bytes(buf[..4].try_into()?))
            }
            protocol_control_type::ABORT => Self::Abort(u32::from_be_bytes(buf[..4].try_into()?)),
            protocol_control_type::ACK => Self::Ack(u32::from_be_bytes(buf[..4].try_into()?)),
            protocol_control_type::WINDOW_ACK_SIZE => {
                Self::AckWindowSize(u32::from_be_bytes(buf[..4].try_into()?))
            }
            protocol_control_type::SET_PEER_BANDWIDTH => Self::SetPeerBandwidth {
                window_size: u32::from_be_bytes(buf[..4].try_into()?),
                limit_type: buf[5],
            },
            _ => panic!(),
        })
    }
}
