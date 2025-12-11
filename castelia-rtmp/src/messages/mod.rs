use crate::messages::{
    command::CommandMessage, protocol_control::ProtolControlMessage,
    user_control::UserControlMessage,
};

pub mod command;
pub mod protocol_control;
pub mod user_control;

#[derive(Debug)]
pub enum Message {
    Protocol(ProtolControlMessage),
    UserControl(UserControlMessage),
    Command(CommandMessage),
}

impl Message {
    pub fn parse_message(_buf: &[u8], message_type: u8) -> Self {
        todo!()
    }
}
