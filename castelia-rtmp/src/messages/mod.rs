#[derive(Debug)]
pub enum Message {
    Protocol,
    UserControl,
    Command,
}

impl Message {
    pub fn parse_message(_buf: &[u8]) -> Self {
        Self::Protocol
    }
}
