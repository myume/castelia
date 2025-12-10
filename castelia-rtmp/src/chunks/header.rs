use std::io;

use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, BufReader},
    net::TcpStream,
};

pub struct ChunkHeader {
    basic_header: BasicHeader,
    message_header: MessageHeader,
    extended_timestamp: Option<u32>,
}

#[derive(Error, Debug)]
pub enum ParseChunkHeaderError {
    #[error("Failed to read chunk header")]
    ReadError(
        #[source]
        #[from]
        io::Error,
    ),
}

enum MessageHeader {
    Type0 {
        timestamp: u32,

        message_length: u32,
        message_type_id: u8,
        message_stream_id: u32,
    },
    Type1 {
        timestamp_delta: u32,

        message_length: u32,
        message_type_id: u8,
    },
    Type2 {
        timestamp_delta: u32,
    },
    Type3,
}

// Uses 2 extra bytes compared to the actual representation.
// We could store the three bytes of the basic header directly, but we'd need to compose and
// translate the bytes into actual values when fetching the value in getters instead.
//
// We will revisit this if it's an issue.
// We are trading some extra space for less conversions between bytes to a u32
struct BasicHeader {
    chunk_type: u8,
    chunk_stream_id: u32,
}

impl MessageHeader {
    async fn parse(reader: &mut BufReader<&mut TcpStream>) -> Result<Self, ParseChunkHeaderError> {
        todo!()
    }
}

impl BasicHeader {
    pub fn chunk_type(&self) -> u8 {
        self.chunk_type
    }

    pub fn chunk_stream_id(&self) -> u32 {
        self.chunk_stream_id
    }

    async fn parse(reader: &mut BufReader<&mut TcpStream>) -> Result<Self, ParseChunkHeaderError> {
        let byte1 = reader.read_u8().await?;

        // bottom 6 bits is header type if 0 or 1 else it's the actual cs_id
        let header_type = byte1 & 0x3F;
        let chunk_stream_id = match header_type {
            // 2 byte form
            0 => {
                let byte2 = reader.read_u8().await?;
                byte2 as u32 + 64
            }
            // 3 byte form
            1 => {
                let byte2 = reader.read_u8().await?;
                let byte3 = reader.read_u8().await?;
                (((byte3 as u16) << 8) + (byte2 as u16 + 64)).into()
            }
            _ => header_type.into(),
        };

        Ok(Self {
            chunk_type: byte1 >> 6,
            chunk_stream_id,
        })
    }
}

impl ChunkHeader {
    pub async fn read_header(
        reader: &mut BufReader<&mut TcpStream>,
    ) -> Result<Self, ParseChunkHeaderError> {
        let basic_header = BasicHeader::parse(reader).await?;
        let message_header = MessageHeader::parse(reader).await?;
        Ok(Self {
            basic_header,
            message_header,
            extended_timestamp: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use tokio::{io::AsyncWriteExt, net::TcpListener};

    use super::*;

    async fn setup(bytes: &[u8]) -> TcpStream {
        let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut client = TcpStream::connect(server.local_addr().unwrap())
            .await
            .unwrap();

        let (stream, _) = server.accept().await.unwrap();
        client.write_all(bytes).await.unwrap();

        stream
    }

    #[tokio::test]
    async fn test_parse_header_one_byte() {
        let bytes = [0b01_000011];
        let mut stream = setup(&bytes).await;
        let mut reader = BufReader::new(&mut stream);
        let header = BasicHeader::parse(&mut reader)
            .await
            .expect("should return header");

        assert_eq!(header.chunk_type(), 1);
        assert_eq!(header.chunk_stream_id(), 3);
    }

    #[tokio::test]
    async fn test_parse_header_two_bytes() {
        let bytes = [0b10 << 6, 200];
        let mut stream = setup(&bytes).await;
        let mut reader = BufReader::new(&mut stream);
        let header = BasicHeader::parse(&mut reader)
            .await
            .expect("should return header");

        assert_eq!(header.chunk_type(), 2);
        assert_eq!(header.chunk_stream_id(), 264);
    }

    #[tokio::test]
    async fn test_parse_header_three_bytes() {
        // 365 to hex is 0x12d, big endian is just 0x2d and 0x01
        let bytes = [0x01, 0x2d, 0x1];
        let mut stream = setup(&bytes).await;
        let mut reader = BufReader::new(&mut stream);
        let header = BasicHeader::parse(&mut reader)
            .await
            .expect("should return header");

        assert_eq!(header.chunk_type(), 0);
        assert_eq!(header.chunk_stream_id(), 365);
    }
}
