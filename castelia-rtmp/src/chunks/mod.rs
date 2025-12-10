use tokio::{io::BufReader, net::TcpStream};

use crate::{chunks::header::ChunkHeader, messages::RTMPMessage};

mod header;

pub struct Chunk {
    pub header: ChunkHeader,
    pub message: RTMPMessage,
}

impl Chunk {
    /// Read a Chunk from the stream
    pub async fn read_chunk(reader: &mut BufReader<&mut TcpStream>) -> Self {
        Self {
            header: ChunkHeader::read_header(reader).await.unwrap(),
            message: RTMPMessage::read_message(reader).await,
        }
    }
}
