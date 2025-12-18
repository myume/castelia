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
    id: u32,
    state: NetStreamState,
    stream: Option<Box<dyn BroadcastStreamer + Send>>,
    stream_key: Option<String>,
}

impl NetStream {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            state: NetStreamState::Active,
            stream: None,
            stream_key: None,
        }
    }
}
