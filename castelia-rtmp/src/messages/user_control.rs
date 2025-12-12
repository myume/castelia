use crate::messages::ParseMessageError;

pub const USER_CONTROL_TYPE: u8 = 4;

pub enum ParseError {
    InvalidEventType(u16),
    InvalidMessageSize,
}

impl From<ParseError> for ParseMessageError {
    fn from(value: ParseError) -> Self {
        todo!()
    }
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
            buf[..2]
                .try_into()
                .map_err(|_| ParseError::InvalidMessageSize)?,
        );

        let data = u32::from_be_bytes(
            buf[2..6]
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
                    buf[2..6]
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
