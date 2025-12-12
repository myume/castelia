use crate::messages::ParseMessageError;

pub const USER_CONTROL_TYPE: u8 = 4;

pub enum ParseError {}

impl From<ParseError> for ParseMessageError {
    fn from(value: ParseError) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub enum UserControlMessage {}

impl UserControlMessage {
    pub fn parse_message(buf: &[u8], message_type_id: &u8) -> Result<Self, ParseError> {
        todo!()
    }
}
