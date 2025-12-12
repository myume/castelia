use crate::amf::AMF0Value;

#[derive(Debug)]
pub enum NetConnectionCommandType {
    Connect,
    Call,
    Close,
    CreateStream,
}

impl TryFrom<&str> for NetConnectionCommandType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "connect" => Self::Connect,
            "call" => Self::Call,
            "close" => Self::Close,
            "createStream" => Self::CreateStream,
            e => return Err(format!("Invalid command type: {e}")),
        })
    }
}

#[derive(Debug)]
pub struct NetConnectionCommand<'a> {
    pub command_type: NetConnectionCommandType,
    pub transaction_id: f64,
    pub command_object: AMF0Value<'a>,
}
