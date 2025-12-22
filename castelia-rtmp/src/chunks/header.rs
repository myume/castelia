use std::io;

use bytes::{BufMut, Bytes, BytesMut};
use thiserror::Error;
use tokio::io::{AsyncReadExt, BufReader};
use tracing::trace;

use crate::chunks::CSId;

#[derive(Debug, PartialEq)]
pub struct ChunkHeader {
    pub basic_header: BasicHeader,
    pub message_header: MessageHeader,
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

impl From<ParseChunkHeaderError> for io::Error {
    fn from(value: ParseChunkHeaderError) -> Self {
        match value {
            ParseChunkHeaderError::ReadError(ref error) => io::Error::new(error.kind(), value),
            ParseChunkHeaderError::InvalidChunkType(_) => {
                io::Error::new(io::ErrorKind::InvalidData, value)
            }
        }
    }
}

#[derive(Debug)]
pub struct FullMessageHeader {
    pub timestamp: u32,
    pub extended_timestamp: Option<u32>,
    pub message_length: u32,
    pub message_type_id: u8,
    pub message_stream_id: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MessageHeader {
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
pub struct BasicHeader {
    chunk_type: u8,
    chunk_stream_id: CSId,
    header_type: BasicHeaderType,
}

#[derive(Debug, PartialEq)]
pub enum BasicHeaderType {
    One,
    Two,
    Three,
}

impl MessageHeader {
    pub fn serialize(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        match self {
            MessageHeader::Type0 {
                timestamp,
                message_length,
                message_type_id,
                message_stream_id,
            } => {
                bytes.put(&timestamp.to_be_bytes()[1..]);
                bytes.put(&message_length.to_be_bytes()[1..]);
                bytes.put_u8(*message_type_id);
                bytes.put_u32_le(*message_stream_id);
            }
            MessageHeader::Type1 {
                timestamp_delta,
                message_length,
                message_type_id,
            } => {
                bytes.put(&timestamp_delta.to_be_bytes()[1..]);
                bytes.put(&message_length.to_be_bytes()[1..]);
                bytes.put_u8(*message_type_id);
            }
            MessageHeader::Type2 { timestamp_delta } => {
                bytes.put(&timestamp_delta.to_be_bytes()[1..]);
            }
            MessageHeader::Type3 => {}
        }
        bytes.into()
    }

    pub fn get_type(&self) -> u8 {
        match self {
            MessageHeader::Type0 { .. } => 0,
            MessageHeader::Type1 { .. } => 1,
            MessageHeader::Type2 { .. } => 2,
            MessageHeader::Type3 => 3,
        }
    }
    pub fn get_message_stream_id(&self) -> Option<u32> {
        match *self {
            MessageHeader::Type0 {
                message_stream_id, ..
            } => Some(message_stream_id),
            _ => None,
        }
    }

    pub fn get_message_type_id(&self) -> Option<u8> {
        match *self {
            MessageHeader::Type0 {
                message_type_id, ..
            } => Some(message_type_id),
            MessageHeader::Type1 {
                message_type_id, ..
            } => Some(message_type_id),
            _ => None,
        }
    }

    pub fn has_extended_timestamp(&self) -> bool {
        0xFFFFFF
            == match *self {
                MessageHeader::Type0 { timestamp, .. } => timestamp,
                MessageHeader::Type1 {
                    timestamp_delta, ..
                } => timestamp_delta,
                MessageHeader::Type2 { timestamp_delta } => timestamp_delta,
                MessageHeader::Type3 => return false,
            }
    }

    async fn parse_type0<T>(reader: &mut BufReader<T>) -> Result<Self, ParseChunkHeaderError>
    where
        T: AsyncReadExt + std::marker::Unpin,
    {
        let timestamp = read_3_be_bytes_to_u32(reader).await?;
        let message_length = read_3_be_bytes_to_u32(reader).await?;
        let message_type_id = reader.read_u8().await?;
        let message_stream_id = reader.read_u32_le().await?;

        Ok(Self::Type0 {
            timestamp,
            message_length,
            message_type_id,
            message_stream_id,
        })
    }

    async fn parse_type1<T>(reader: &mut BufReader<T>) -> Result<Self, ParseChunkHeaderError>
    where
        T: AsyncReadExt + std::marker::Unpin,
    {
        let timestamp_delta = read_3_be_bytes_to_u32(reader).await?;
        let message_length = read_3_be_bytes_to_u32(reader).await?;
        let message_type_id = reader.read_u8().await?;
        Ok(Self::Type1 {
            timestamp_delta,
            message_length,
            message_type_id,
        })
    }
    async fn parse_type2<T>(reader: &mut BufReader<T>) -> Result<Self, ParseChunkHeaderError>
    where
        T: AsyncReadExt + std::marker::Unpin,
    {
        Ok(Self::Type2 {
            timestamp_delta: read_3_be_bytes_to_u32(reader).await?,
        })
    }
    async fn parse_type3() -> Result<Self, ParseChunkHeaderError> {
        Ok(Self::Type3)
    }

    async fn parse<T>(
        reader: &mut BufReader<T>,
        header_type: &u8,
    ) -> Result<Self, ParseChunkHeaderError>
    where
        T: AsyncReadExt + std::marker::Unpin,
    {
        trace!("parsing chunk message header");
        match *header_type {
            0 => Self::parse_type0(reader).await,
            1 => Self::parse_type1(reader).await,
            2 => Self::parse_type2(reader).await,
            3 => Self::parse_type3().await,
            e => Err(ParseChunkHeaderError::InvalidChunkType(e)),
        }
    }
}

pub async fn read_3_be_bytes_to_u32<T>(reader: &mut BufReader<T>) -> Result<u32, io::Error>
where
    T: AsyncReadExt + std::marker::Unpin,
{
    Ok(u32::from_be_bytes([
        0x00,
        reader.read_u8().await?,
        reader.read_u8().await?,
        reader.read_u8().await?,
    ]))
}

impl BasicHeader {
    pub fn serialize(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        match self.header_type {
            BasicHeaderType::One => {
                bytes.put_u8(self.chunk_type << 6 | (self.chunk_stream_id as u8 & 0x3F));
            }
            BasicHeaderType::Two => {
                bytes.put_u8(self.chunk_type << 6);
                bytes.put_u8((self.chunk_stream_id - 64) as u8);
            }
            BasicHeaderType::Three => {
                bytes.put_u8(self.chunk_type << 6 | 0x01);
                bytes.put_u8((self.chunk_stream_id >> 8) as u8);
                bytes.put_u8((self.chunk_stream_id - 64) as u8);
            }
        }

        bytes.into()
    }

    pub fn chunk_type(&self) -> u8 {
        self.chunk_type
    }

    pub fn chunk_stream_id(&self) -> u32 {
        self.chunk_stream_id
    }

    async fn parse<T>(reader: &mut BufReader<T>) -> Result<Self, ParseChunkHeaderError>
    where
        T: AsyncReadExt + std::marker::Unpin,
    {
        trace!("parsing chunk basic header");
        let byte1 = reader.read_u8().await?;
        let mut bytes_read = 1;

        // bottom 6 bits is header type if 0 or 1 else it's the actual cs_id
        let chunk_stream_id = match byte1 & 0x3F {
            // 2 byte form
            0 => {
                let byte2 = reader.read_u8().await?;
                bytes_read += 1;
                byte2 as u32 + 64
            }
            // 3 byte form
            1 => {
                let byte2 = reader.read_u8().await?;
                bytes_read += 1;
                let byte3 = reader.read_u8().await?;
                bytes_read += 1;
                (((byte3 as u16) << 8) + (byte2 as u16 + 64)).into()
            }
            val => val.into(),
        };

        Ok(Self {
            chunk_type: byte1 >> 6,
            chunk_stream_id,
            header_type: match bytes_read {
                1 => BasicHeaderType::One,
                2 => BasicHeaderType::Two,
                _ => BasicHeaderType::Three,
            },
        })
    }

    pub fn new(fmt: u8, csid: u32) -> Self {
        let header_type = match csid {
            0..64 => BasicHeaderType::One,
            64..320 => BasicHeaderType::Two,
            _ => BasicHeaderType::Three,
        };
        Self {
            chunk_type: fmt,
            chunk_stream_id: csid,
            header_type,
        }
    }
}

impl ChunkHeader {
    pub fn new(
        basic_header: BasicHeader,
        message_header: MessageHeader,
        extended_timestamp: Option<u32>,
    ) -> Self {
        Self {
            basic_header,
            message_header,
            extended_timestamp,
        }
    }

    pub fn get_message_type(&self) -> Option<u8> {
        self.message_header.get_message_type_id()
    }

    pub fn chunk_stream_id(&self) -> CSId {
        self.basic_header.chunk_stream_id()
    }

    pub fn get_message_stream_id(&self) -> Option<u32> {
        self.message_header.get_message_stream_id()
    }

    pub async fn read_header<T>(reader: &mut BufReader<T>) -> Result<Self, ParseChunkHeaderError>
    where
        T: AsyncReadExt + std::marker::Unpin,
    {
        trace!("reading chunk header");
        let basic_header = BasicHeader::parse(reader).await?;
        let message_header = MessageHeader::parse(reader, &basic_header.chunk_type()).await?;
        let extended_timestamp = if message_header.has_extended_timestamp() {
            trace!("reading chunk extended timestamp");
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

    pub fn get_message_length(&self) -> Option<u32> {
        match self.message_header {
            MessageHeader::Type0 { message_length, .. } => Some(message_length),
            MessageHeader::Type1 { message_length, .. } => Some(message_length),
            MessageHeader::Type2 { .. } => None,
            MessageHeader::Type3 => None,
        }
    }

    pub fn serialize(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        bytes.put(self.basic_header.serialize());
        bytes.put(self.message_header.serialize());
        if let Some(extended_timestamp) = self.extended_timestamp {
            bytes.put_u32(extended_timestamp);
        }
        bytes.into()
    }
}

#[cfg(test)]
mod tests {
    use tokio::{
        io::AsyncWriteExt,
        net::{TcpListener, TcpStream},
    };

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
                message_stream_id: 0xefcdab10 // message stream id is in little endian
            }
        );
        assert!(!header.has_extended_timestamp());
    }

    #[tokio::test]
    async fn test_serialize_and_parse_basic_header() {
        let expected = BasicHeader::new(0, 2);
        let bytes = expected.serialize();
        let mut reader: BufReader<&[u8]> = BufReader::new(&bytes);
        let actual = BasicHeader::parse(&mut reader).await.unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_serialize_and_parse_message_header_type0() {
        let expected = MessageHeader::Type0 {
            timestamp: rand::random::<u16>().into(),
            message_length: rand::random::<u16>().into(),
            message_type_id: rand::random(),
            message_stream_id: rand::random(),
        };
        let bytes = expected.serialize();
        let mut reader: BufReader<&[u8]> = BufReader::new(&bytes);
        let actual = MessageHeader::parse(&mut reader, &expected.get_type())
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_serialize_and_parse_message_header_type1() {
        let expected = MessageHeader::Type1 {
            timestamp_delta: rand::random::<u16>().into(),
            message_length: rand::random::<u16>().into(),
            message_type_id: rand::random(),
        };
        let bytes = expected.serialize();
        let mut reader: BufReader<&[u8]> = BufReader::new(&bytes);
        let actual = MessageHeader::parse(&mut reader, &expected.get_type())
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_serialize_and_parse_message_header_type2() {
        let expected = MessageHeader::Type2 {
            timestamp_delta: rand::random::<u16>().into(),
        };
        let bytes = expected.serialize();
        let mut reader: BufReader<&[u8]> = BufReader::new(&bytes);
        let actual = MessageHeader::parse(&mut reader, &expected.get_type())
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_serialize_and_parse_message_header_type3() {
        let expected = MessageHeader::Type3;
        let bytes = expected.serialize();
        let mut reader: BufReader<&[u8]> = BufReader::new(&bytes);
        let actual = MessageHeader::parse(&mut reader, &expected.get_type())
            .await
            .unwrap();
        assert_eq!(expected, actual);
    }
}
