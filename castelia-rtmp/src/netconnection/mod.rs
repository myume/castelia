pub mod command;
pub mod handler;

#[derive(Debug)]
pub struct NetConnection {
    max_chunk_size: u32,
}

impl NetConnection {
    pub fn new() -> Self {
        NetConnection {
            max_chunk_size: 4096,
        }
    }

    pub fn max_chunk_size(&self) -> u32 {
        self.max_chunk_size
    }
}
