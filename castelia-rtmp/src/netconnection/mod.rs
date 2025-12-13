use crate::messages::Message;

#[derive(Debug)]
pub enum NetConnectionCommandType<'a> {
    Connect,
    Call(&'a str),
    Close,
    CreateStream,
}

impl<'a> From<&'a str> for NetConnectionCommandType<'a> {
    fn from(value: &'a str) -> Self {
        match value {
            "connect" => Self::Connect,
            "close" => Self::Close,
            "createStream" => Self::CreateStream,
            procedure_name => Self::Call(procedure_name),
        }
    }
}

#[derive(Debug)]
pub struct NetConnection {
    max_chunk_size: u32,
}

impl NetConnection {
    pub fn new() -> Self {
        NetConnection {
            max_chunk_size: 4096,
        }
    }

    pub fn max_chunk_size(&self) -> u32 {
        self.max_chunk_size
    }

    pub fn handle_message() {}
}
