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
    pub id: u32,
    state: NetStreamState,
    stream: Option<Box<dyn BroadcastStreamer + Send>>,
    pub stream_key: Option<String>,
    audio_header_sent: bool,
    video_header_sent: bool,
}

impl NetStream {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            state: NetStreamState::Active,
            stream: None,
            stream_key: None,
            audio_header_sent: false,
            video_header_sent: false,
        }
    }
}
