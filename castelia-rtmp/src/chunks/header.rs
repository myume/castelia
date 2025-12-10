use std::io;

use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, BufReader},
    net::TcpStream,
};

#[derive(Debug, PartialEq)]
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
    #[error("Invalid chunk type found: {0}")]
    InvalidChunkType(u8),
}

#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
struct BasicHeader {
    chunk_type: u8,
    chunk_stream_id: u32,
}

impl MessageHeader {
    pub fn has_extended_timestamp(&self) -> bool {
        0xFFFFFF
            == match *self {
                MessageHeader::Type0 {
                    timestamp,
                    message_length: _,
                    message_type_id: _,
                    message_stream_id: _,
                } => timestamp,
                MessageHeader::Type1 {
                    timestamp_delta,
                    message_length: _,
                    message_type_id: _,
                } => timestamp_delta,
                MessageHeader::Type2 { timestamp_delta } => timestamp_delta,
                MessageHeader::Type3 => return false,
            }
    }

    async fn parse_type0(
        reader: &mut BufReader<&mut TcpStream>,
    ) -> Result<Self, ParseChunkHeaderError> {
        let timestamp = read_3_be_bytes_to_u32(reader).await?;
        let message_length = read_3_be_bytes_to_u32(reader).await?;
        let message_type_id = reader.read_u8().await?;
        let message_stream_id = reader.read_u32().await?;

        Ok(Self::Type0 {
            timestamp,
            message_length,
            message_type_id,
            message_stream_id,
        })
    }

    async fn parse_type1(
        reader: &mut BufReader<&mut TcpStream>,
    ) -> Result<Self, ParseChunkHeaderError> {
        let timestamp_delta = read_3_be_bytes_to_u32(reader).await?;
        let message_length = read_3_be_bytes_to_u32(reader).await?;
        let message_type_id = reader.read_u8().await?;
        Ok(Self::Type1 {
            timestamp_delta,
            message_length,
            message_type_id,
        })
    }
    async fn parse_type2(
        reader: &mut BufReader<&mut TcpStream>,
    ) -> Result<Self, ParseChunkHeaderError> {
        Ok(Self::Type2 {
            timestamp_delta: read_3_be_bytes_to_u32(reader).await?,
        })
    }
    async fn parse_type3() -> Result<Self, ParseChunkHeaderError> {
        Ok(Self::Type3)
    }

    async fn parse(
        reader: &mut BufReader<&mut TcpStream>,
        chunk_type: &u8,
    ) -> Result<Self, ParseChunkHeaderError> {
        match *chunk_type {
            0 => Self::parse_type0(reader).await,
            1 => Self::parse_type1(reader).await,
            2 => Self::parse_type2(reader).await,
            3 => Self::parse_type3().await,
            e => Err(ParseChunkHeaderError::InvalidChunkType(e)),
        }
    }
}

pub async fn read_3_be_bytes_to_u32(
    reader: &mut BufReader<&mut TcpStream>,
) -> Result<u32, io::Error> {
    Ok(u32::from_be_bytes([
        0x00,
        reader.read_u8().await?,
        reader.read_u8().await?,
        reader.read_u8().await?,
    ]))
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
        let message_header = MessageHeader::parse(reader, &basic_header.chunk_type()).await?;
        let extended_timestamp = if message_header.has_extended_timestamp() {
            Some(reader.read_u32().await?)
        } else {
            None
        };

        Ok(Self {
            basic_header,
            message_header,
            extended_timestamp,
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

    #[tokio::test]
    async fn test_3be_bytes_to_u32() {
        let expected: u32 = rand::random();
        let mut stream = setup(&expected.to_be_bytes()[1..]).await;
        let mut reader = BufReader::new(&mut stream);

        let result = read_3_be_bytes_to_u32(&mut reader)
            .await
            .expect("read should succeed");

        assert_eq!(
            result & 0xFFFFFF,
            expected & 0xFFFFFF,
            "found: {:#08x}, expected: {:#08x}",
            result,
            expected,
        );
    }

    #[tokio::test]
    async fn test_parse_message_header_type3() {
        let bytes = [0x01, 0x2d, 0x1];
        let mut stream = setup(&bytes).await;
        let mut reader = BufReader::new(&mut stream);
        let header = MessageHeader::parse(&mut reader, &3)
            .await
            .expect("should return header");

        assert_eq!(header, MessageHeader::Type3);
        assert!(!header.has_extended_timestamp());
    }

    #[tokio::test]
    async fn test_parse_message_header_type2() {
        let bytes = [0x12, 0x34, 0x56];
        let mut stream = setup(&bytes).await;
        let mut reader = BufReader::new(&mut stream);
        let header = MessageHeader::parse(&mut reader, &2)
            .await
            .expect("should return header");

        assert_eq!(
            header,
            MessageHeader::Type2 {
                timestamp_delta: 0x123456
            }
        );
        assert!(!header.has_extended_timestamp());
    }

    #[tokio::test]
    async fn test_parse_message_header_type1() {
        let bytes = [
            0x12, 0x34, 0x56, // delta
            0x11, 0x22, 0x33, // length
            0xcd, // message type id
        ];
        let mut stream = setup(&bytes).await;
        let mut reader = BufReader::new(&mut stream);
        let header = MessageHeader::parse(&mut reader, &1)
            .await
            .expect("should return header");

        assert_eq!(
            header,
            MessageHeader::Type1 {
                timestamp_delta: 0x123456,
                message_length: 0x112233,
                message_type_id: 0xcd
            }
        );
        assert!(!header.has_extended_timestamp());
    }

    #[tokio::test]
    async fn test_parse_message_header_type0() {
        let bytes = [
            0x12, 0x34, 0x56, // timestamp
            0x11, 0x22, 0x33, // length
            0xcd, // message type id
            0x10, 0xab, 0xcd, 0xef, // message stream id
        ];
        let mut stream = setup(&bytes).await;
        let mut reader = BufReader::new(&mut stream);
        let header = MessageHeader::parse(&mut reader, &0)
            .await
            .expect("should return header");

        assert_eq!(
            header,
            MessageHeader::Type0 {
                timestamp: 0x123456,
                message_length: 0x112233,
                message_type_id: 0xcd,
                message_stream_id: 0x10abcdef
            }
        );
        assert!(!header.has_extended_timestamp());
    }
}
