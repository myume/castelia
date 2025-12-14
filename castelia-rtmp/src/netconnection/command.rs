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
