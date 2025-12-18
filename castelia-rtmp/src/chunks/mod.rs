use bytes::{BufMut, Bytes, BytesMut};

use crate::chunks::header::{BasicHeader, ChunkHeader, FullMessageHeader, MessageHeader};

pub mod chunk_handler;
pub mod header;

type CSId = u32;

pub(crate) const SERVER_CHUNK_SIZE: u32 = 4096;

#[derive(Debug, PartialEq)]
pub struct Chunk {
    pub header: ChunkHeader,
    pub payload: Bytes,
}

impl Chunk {
    /// Break the payload into chunks
    pub fn into_chunks(
        full_header: &FullMessageHeader,
        mut payload: Bytes,
        chunk_size: u32,
    ) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        // we will naively chunk the payload like this:
        // first chunk will always have a full header
        // everything after will by of type 3
        while !payload.is_empty() {
            let chunked_payload = payload.split_to((chunk_size as usize).min(payload.len()));
            let message_header = if chunks.is_empty() {
                MessageHeader::Type0 {
                    timestamp: full_header.timestamp,
                    message_length: full_header.message_length,
                    message_type_id: full_header.message_type_id,
                    message_stream_id: full_header.message_stream_id,
                }
            } else {
                MessageHeader::Type3
            };

            let csid = match full_header.message_type_id {
                1..=6 => 2,
                8 => 4,
                9 => 5,
                20 => 3,
                _ => 6,
            };
            let basic_header = BasicHeader::new(message_header.get_type(), csid);

            let header =
                ChunkHeader::new(basic_header, message_header, full_header.extended_timestamp);

            let chunk = Chunk {
                header,
                payload: chunked_payload,
            };
            chunks.push(chunk);
        }

        chunks
    }

    pub fn serialize(&self) -> Bytes {
        let mut bytes = BytesMut::new();
        bytes.put(self.header.serialize());
        bytes.put(self.payload.clone());
        bytes.into()
    }
}

#[cfg(test)]
mod tests {
    use std::iter::zip;

    use super::*;

    #[test]
    fn test_chunk_payload() {
        let mut payload = [0; 3 * SERVER_CHUNK_SIZE as usize + 128];
        rand::fill(&mut payload);

        let full_header = FullMessageHeader {
            timestamp: 0,
            extended_timestamp: None,
            message_length: payload.len() as u32,
            message_type_id: 1,
            message_stream_id: 0,
        };

        let expected: Vec<Chunk> = vec![
            Chunk {
                header: ChunkHeader::new(
                    BasicHeader::new(0, 2),
                    MessageHeader::Type0 {
                        timestamp: full_header.timestamp,
                        message_length: full_header.message_length,
                        message_type_id: full_header.message_type_id,
                        message_stream_id: full_header.message_stream_id,
                    },
                    None,
                ),
                payload: Bytes::from(payload[0..SERVER_CHUNK_SIZE as usize].to_vec()),
            },
            Chunk {
                header: ChunkHeader::new(BasicHeader::new(3, 2), MessageHeader::Type3, None),
                payload: Bytes::from(
                    payload[SERVER_CHUNK_SIZE as usize..2 * SERVER_CHUNK_SIZE as usize].to_vec(),
                ),
            },
            Chunk {
                header: ChunkHeader::new(BasicHeader::new(3, 2), MessageHeader::Type3, None),
                payload: Bytes::from(
                    payload[2 * SERVER_CHUNK_SIZE as usize..3 * SERVER_CHUNK_SIZE as usize]
                        .to_vec(),
                ),
            },
            Chunk {
                header: ChunkHeader::new(BasicHeader::new(3, 2), MessageHeader::Type3, None),
                payload: Bytes::from(payload[3 * SERVER_CHUNK_SIZE as usize..].to_vec()),
            },
        ];
        let chunks =
            Chunk::into_chunks(&full_header, Bytes::from_owner(payload), SERVER_CHUNK_SIZE);

        // asserting like this for easier debugging
        for (expected, actual) in zip(&expected, &chunks) {
            assert_eq!(expected.header, actual.header);
            assert_eq!(expected.payload, actual.payload);
        }
        assert_eq!(expected, chunks)
    }
}
