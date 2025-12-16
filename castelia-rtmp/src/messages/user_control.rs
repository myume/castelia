use bytes::{BufMut, Bytes, BytesMut};
use thiserror::Error;

pub const USER_CONTROL_TYPE: u8 = 4;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid event type {0}")]
    InvalidEventType(u16),
    #[error("Invalid message size")]
    InvalidMessageSize,
}

#[derive(Debug, PartialEq)]
pub enum UserControlMessage {
    StreamBegin(u32),
    StreamEOF(u32),
    StreamDry(u32),
    SetBufferLength {
        message_stream_id: u32,
        buffer_size_in_millis: u32,
    },
    StreamIsRecord(u32),
    PingRequest(u32),
    PingRepsonse(u32),
}

impl UserControlMessage {
    pub fn parse_message(buf: &[u8]) -> Result<Self, ParseError> {
        let event_type = u16::from_be_bytes(
            buf.get(..2)
                .ok_or(ParseError::InvalidMessageSize)?
                .try_into()
                .map_err(|_| ParseError::InvalidMessageSize)?,
        );

        let data = u32::from_be_bytes(
            buf.get(2..6)
                .ok_or(ParseError::InvalidMessageSize)?
                .try_into()
                .map_err(|_| ParseError::InvalidMessageSize)?,
        );

        Ok(match event_type {
            0 => Self::StreamBegin(data),
            1 => Self::StreamEOF(data),
            2 => Self::StreamDry(data),
            3 => Self::SetBufferLength {
                message_stream_id: data,
                buffer_size_in_millis: u32::from_be_bytes(
                    buf.get(6..10)
                        .ok_or(ParseError::InvalidMessageSize)?
                        .try_into()
                        .map_err(|_| ParseError::InvalidMessageSize)?,
                ),
            },
            4 => Self::StreamIsRecord(data),
            5 => Self::PingRequest(data),
            6 => Self::PingRepsonse(data),
            _ => return Err(ParseError::InvalidEventType(event_type)),
        })
    }

    pub fn serialize(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        match self {
            UserControlMessage::StreamBegin(data) => {
                bytes.put_u16(0);
                bytes.extend_from_slice(&data.to_be_bytes());
            }
            UserControlMessage::StreamEOF(data) => {
                bytes.put_u16(1);
                bytes.extend_from_slice(&data.to_be_bytes());
            }
            UserControlMessage::StreamDry(data) => {
                bytes.put_u16(2);
                bytes.extend_from_slice(&data.to_be_bytes());
            }
            UserControlMessage::SetBufferLength {
                message_stream_id,
                buffer_size_in_millis,
            } => {
                bytes.put_u16(3);
                bytes.extend_from_slice(&message_stream_id.to_be_bytes());
                bytes.extend_from_slice(&buffer_size_in_millis.to_be_bytes());
            }
            UserControlMessage::StreamIsRecord(data) => {
                bytes.put_u16(4);
                bytes.extend_from_slice(&data.to_be_bytes());
            }
            UserControlMessage::PingRequest(data) => {
                bytes.put_u16(5);
                bytes.extend_from_slice(&data.to_be_bytes());
            }
            UserControlMessage::PingRepsonse(data) => {
                bytes.put_u16(6);
                bytes.extend_from_slice(&data.to_be_bytes());
            }
        }

        bytes.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_begin() {
        let expected = UserControlMessage::StreamBegin(rand::random());
        let bytes = expected.serialize();
        let actual = UserControlMessage::parse_message(&bytes).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_stream_eof() {
        let expected = UserControlMessage::StreamEOF(rand::random());
        let bytes = expected.serialize();
        let actual = UserControlMessage::parse_message(&bytes).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_stream_dry() {
        let expected = UserControlMessage::StreamDry(rand::random());
        let bytes = expected.serialize();
        let actual = UserControlMessage::parse_message(&bytes).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_stream_is_record() {
        let expected = UserControlMessage::StreamIsRecord(rand::random());
        let bytes = expected.serialize();
        let actual = UserControlMessage::parse_message(&bytes).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_ping_request() {
        let expected = UserControlMessage::PingRequest(rand::random());
        let bytes = expected.serialize();
        let actual = UserControlMessage::parse_message(&bytes).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_ping_response() {
        let expected = UserControlMessage::PingRepsonse(rand::random());
        let bytes = expected.serialize();
        let actual = UserControlMessage::parse_message(&bytes).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_set_buffer_length() {
        let expected = UserControlMessage::SetBufferLength {
            message_stream_id: rand::random(),
            buffer_size_in_millis: rand::random(),
        };
        let bytes = expected.serialize();
        let actual = UserControlMessage::parse_message(&bytes).unwrap();
        assert_eq!(expected, actual);
    }
}
