use crate::amf::AMF0Value;

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
pub struct NetConnectionCommand<'a> {
    pub command_type: NetConnectionCommandType<'a>,
    pub transaction_id: f64,
    pub command_object: AMF0Value<'a>,
}
