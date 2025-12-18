use std::fmt::Display;

use crate::amf;

#[derive(Debug, PartialEq)]
pub enum NetConnectionCommandType<'a> {
    Connect,
    Call(&'a str),
    Close,
    CreateStream,
}

impl<'a> Display for NetConnectionCommandType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                NetConnectionCommandType::Connect => "connect",
                NetConnectionCommandType::Call(procedure) => procedure,
                NetConnectionCommandType::Close => "close",
                NetConnectionCommandType::CreateStream => "createStream",
            }
        )
    }
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

#[derive(Debug, PartialEq)]
pub struct NetConnectionCommand<'a> {
    pub command_type: NetConnectionCommandType<'a>,
    pub transaction_id: f64,
    pub command_object: amf::AMF0Value<'a>,
}
