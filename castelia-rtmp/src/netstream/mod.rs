pub mod command;
pub mod handler;

#[derive(Debug)]
pub struct NetStream {}

impl NetStream {
    pub fn new() -> Self {
        Self {}
    }
}
