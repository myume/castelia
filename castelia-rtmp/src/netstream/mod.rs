pub mod command;
pub mod handler;

#[derive(Debug)]
enum NetStreamState {
    Active,
    Publishing,
    Closed,
}

#[derive(Debug)]
pub struct NetStream {
    state: NetStreamState,
}

impl NetStream {
    pub fn new() -> Self {
        Self {
            state: NetStreamState::Active,
        }
    }
}
