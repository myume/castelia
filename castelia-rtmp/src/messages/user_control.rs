pub const USER_CONTROL_TYPE: u8 = 4;

#[derive(Debug)]
pub enum UserControlMessage {}

impl UserControlMessage {
    pub fn parse_message(buf: &[u8]) -> Self {
        todo!()
    }
}
