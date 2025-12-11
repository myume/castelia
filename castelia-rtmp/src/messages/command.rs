#[derive(Debug)]
pub enum CommandMessage {
    Command,
    Data,
    SharedObject,
    Audio,
    Video,
    Aggregate,
}

impl CommandMessage {
    pub fn parse_message(buf: &[u8]) -> Self {
        todo!()
    }
}
