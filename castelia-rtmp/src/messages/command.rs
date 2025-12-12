pub mod command_message_type {
    pub const COMMAND_AMF0: u8 = 20;
    pub const COMMAND_AMF3: u8 = 17;
    pub const DATA_AMF0: u8 = 18;
    pub const DATA_AMF3: u8 = 15;
    pub const SHARED_OBJECT_AMF0: u8 = 16;
    pub const SHARED_OBJECT_AMF3: u8 = 14;
    pub const AUDIO: u8 = 8;
    pub const VIDEO: u8 = 9;
    pub const AGGREGATE: u8 = 22;
}

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
