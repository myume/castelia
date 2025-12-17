use crate::broadcast::BroadcastStreamer;

pub mod command;
pub mod handler;

#[derive(Debug)]
enum NetStreamState {
    Active,
    Publishing,
    Closed,
}

pub struct NetStream {
    state: NetStreamState,
    stream: Option<Box<dyn BroadcastStreamer + Send>>,
}

impl NetStream {
    pub fn new() -> Self {
        Self {
            state: NetStreamState::Active,
            stream: None,
        }
    }
}
