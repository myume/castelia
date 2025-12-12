use thiserror::Error;

pub const USER_CONTROL_TYPE: u8 = 4;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid event type {0}")]
    InvalidEventType(u16),
    #[error("Invalid message size")]
    InvalidMessageSize,
}

#[derive(Debug)]
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
                    buf.get(2..6)
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
}
