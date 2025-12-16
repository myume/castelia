use bytes::{BufMut, Bytes, BytesMut};
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

#[derive(Debug, PartialEq)]
pub struct PeerBandwidth {
    pub limit_type: u8,
    pub window_size: u32,
}

#[derive(Debug, PartialEq)]
pub enum ProtocolControlMessage {
    SetChunkSize(u32),
    Abort(u32),
    Ack(u32),
    AckWindowSize(u32),
    SetPeerBandwidth(PeerBandwidth),
}

impl ProtocolControlMessage {
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
            protocol_control_type::SET_PEER_BANDWIDTH => Self::SetPeerBandwidth(PeerBandwidth {
                window_size: data,
                limit_type: *buf.get(4).ok_or(ParseError::InvalidMessageSize)?,
            }),
            _ => return Err(ParseError::InvalidMessageTypeId(*message_type_id)),
        })
    }

    pub fn serialize(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        match self {
            ProtocolControlMessage::SetChunkSize(data) => {
                bytes.extend_from_slice(&data.to_be_bytes());
            }
            ProtocolControlMessage::Abort(data) => {
                bytes.extend_from_slice(&data.to_be_bytes());
            }
            ProtocolControlMessage::Ack(data) => {
                bytes.extend_from_slice(&data.to_be_bytes());
            }
            ProtocolControlMessage::AckWindowSize(size) => {
                bytes.extend_from_slice(&size.to_be_bytes());
            }
            ProtocolControlMessage::SetPeerBandwidth(peer_bandwidth) => {
                bytes.extend_from_slice(&peer_bandwidth.window_size.to_be_bytes());
                bytes.put_u8(peer_bandwidth.limit_type);
            }
        }
        bytes.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_set_chunk_size() {
        let expected = ProtocolControlMessage::SetChunkSize(rand::random());
        let bytes = expected.serialize();
        let actual =
            ProtocolControlMessage::parse_message(&bytes, &protocol_control_type::SET_CHUNK_SIZE)
                .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_abort() {
        let expected = ProtocolControlMessage::Abort(rand::random());
        let bytes = expected.serialize();
        let actual =
            ProtocolControlMessage::parse_message(&bytes, &protocol_control_type::ABORT).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_ack() {
        let expected = ProtocolControlMessage::Ack(rand::random());
        let bytes = expected.serialize();
        let actual =
            ProtocolControlMessage::parse_message(&bytes, &protocol_control_type::ACK).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_ack_window_size() {
        let expected = ProtocolControlMessage::AckWindowSize(rand::random());
        let bytes = expected.serialize();
        let actual =
            ProtocolControlMessage::parse_message(&bytes, &protocol_control_type::WINDOW_ACK_SIZE)
                .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_peer_bandwidth() {
        let expected = ProtocolControlMessage::SetPeerBandwidth(PeerBandwidth {
            limit_type: rand::random(),
            window_size: rand::random(),
        });
        let bytes = expected.serialize();
        let actual = ProtocolControlMessage::parse_message(
            &bytes,
            &protocol_control_type::SET_PEER_BANDWIDTH,
        )
        .unwrap();
        assert_eq!(expected, actual);
    }
}
