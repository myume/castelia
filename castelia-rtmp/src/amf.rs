// A naive amf0 parser
// implemented the bare minimum to parse amf0 for the rtmp protocol
// seems like not the full specification/all the types are used in the protocol

use std::{
    collections::HashMap,
    io::{Cursor, Seek},
    str, vec,
};

use thiserror::Error;

mod amf0_type_marker {
    pub const NUMBER: u8 = 0x00;
    pub const BOOL: u8 = 0x01;
    pub const STRING: u8 = 0x02;
    pub const OBJECT_START: u8 = 0x03;

    pub const ECMA_ARRAY: u8 = 0x08;
    pub const NULL: u8 = 0x05;

    // needs to be preceeded by 2 0x00s
    // so actual object end is 0x00, 0x00, 0x09
    const OBJECT_END: u8 = 0x09;
    pub const OBJECT_END_MARKER: [u8; 3] = [0x00, 0x00, OBJECT_END];
}

#[derive(Debug, PartialEq)]
pub enum AMF0Value<'a> {
    Number(f64),
    Boolean(bool),
    String(&'a str),
    Object(HashMap<&'a str, AMF0Value<'a>>),
    EcmaArray(HashMap<&'a str, AMF0Value<'a>>),
    Null,
}

impl<'a> AMF0Value<'a> {
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            AMF0Value::Number(num) => {
                let mut bytes = vec![amf0_type_marker::NUMBER];
                bytes.extend_from_slice(&num.to_be_bytes());
                bytes
            }
            AMF0Value::Boolean(val) => vec![amf0_type_marker::BOOL, if *val { 0x01 } else { 0x00 }],
            AMF0Value::String(s) => {
                let mut bytes = vec![amf0_type_marker::STRING];
                bytes.extend_from_slice(&(s.len() as u16).to_be_bytes());
                bytes.extend_from_slice(s.as_bytes());
                bytes
            }
            AMF0Value::Object(object) => {
                let mut bytes = vec![amf0_type_marker::OBJECT_START];
                for (key, value) in object {
                    bytes.extend_from_slice(&(key.len() as u16).to_be_bytes());
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.extend_from_slice(&value.serialize());
                }
                bytes.extend_from_slice(&amf0_type_marker::OBJECT_END_MARKER);
                bytes
            }
            AMF0Value::Null => vec![amf0_type_marker::NULL],
            AMF0Value::EcmaArray(amf0_values) => {
                let mut bytes = vec![amf0_type_marker::ECMA_ARRAY];
                bytes.extend_from_slice(&(amf0_values.len() as u32).to_be_bytes());
                for (key, value) in amf0_values {
                    bytes.extend_from_slice(&(key.len() as u16).to_be_bytes());
                    bytes.extend_from_slice(key.as_bytes());
                    bytes.extend_from_slice(&value.serialize());
                }
                bytes.extend_from_slice(&amf0_type_marker::OBJECT_END_MARKER);
                bytes
            }
        }
    }
}

impl<'a> TryFrom<AMF0Value<'a>> for &'a str {
    type Error = CastError;

    fn try_from(value: AMF0Value<'a>) -> Result<Self, Self::Error> {
        match value {
            AMF0Value::String(s) => Ok(s),
            found => Err(CastError::TypeMismatch(format!(
                "Expected string, found {found:?}"
            ))),
        }
    }
}

impl<'a> TryFrom<AMF0Value<'a>> for f64 {
    type Error = CastError;

    fn try_from(value: AMF0Value<'a>) -> Result<Self, Self::Error> {
        match value {
            AMF0Value::Number(num) => Ok(num),
            found => Err(CastError::TypeMismatch(format!(
                "Expected number, found {found:?}"
            ))),
        }
    }
}

impl<'a> TryFrom<AMF0Value<'a>> for bool {
    type Error = CastError;

    fn try_from(value: AMF0Value<'a>) -> Result<Self, Self::Error> {
        match value {
            AMF0Value::Boolean(b) => Ok(b),
            found => Err(CastError::TypeMismatch(format!(
                "Expected bool, found {found:?}"
            ))),
        }
    }
}

#[derive(Debug, Error)]
pub enum CastError {
    #[error("{0}")]
    TypeMismatch(String),
}

#[derive(Debug, Error, PartialEq)]
pub enum DecodeError {
    #[error("Invalid AMF message size")]
    UnexpectedEOF,
    #[error("Unknown marker {0:#04x}")]
    UnknownMarker(u8),
    #[error("String contains invalid utf8")]
    InvalidUtf8(#[from] str::Utf8Error),
    #[error("Invalid object key")]
    InvalidObjectKey,
    #[error("Invalid number")]
    InvalidNumber,
    #[error("Invalid bool")]
    InvalidBool,
}

pub struct Decoder<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> Decoder<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(buf),
        }
    }

    pub fn get_buf(&self) -> Result<&'a [u8], DecodeError> {
        self.cursor
            .get_ref()
            .get(self.cursor.position() as usize..)
            .ok_or(DecodeError::UnexpectedEOF)
    }

    pub fn decode(&mut self) -> Result<AMF0Value<'a>, DecodeError> {
        let type_marker = self.get_buf()?.first().ok_or(DecodeError::UnexpectedEOF)?;
        self.cursor
            .seek_relative(1)
            .map_err(|_| DecodeError::UnexpectedEOF)?;
        let value = match *type_marker {
            amf0_type_marker::NUMBER => self.decode_number()?,
            amf0_type_marker::BOOL => self.decode_bool()?,
            amf0_type_marker::STRING => self.decode_string()?,
            amf0_type_marker::OBJECT_START => self.decode_object()?,
            amf0_type_marker::NULL => AMF0Value::Null,
            amf0_type_marker::ECMA_ARRAY => self.decode_ecma_array()?,
            marker => return Err(DecodeError::UnknownMarker(marker)),
        };

        Ok(value)
    }

    fn decode_ecma_array(&mut self) -> Result<AMF0Value<'a>, DecodeError> {
        // skip 4 byte count
        self.cursor
            .seek_relative(4)
            .map_err(|_| DecodeError::UnexpectedEOF)?;

        Ok(AMF0Value::EcmaArray(self.read_kv_pairs()?))
    }

    fn decode_number(&mut self) -> Result<AMF0Value<'a>, DecodeError> {
        let number_size = 8;
        let number = f64::from_be_bytes(
            self.get_buf()?
                .get(..number_size)
                .ok_or(DecodeError::InvalidNumber)?
                .try_into()
                .map_err(|_| DecodeError::UnexpectedEOF)?,
        );
        self.cursor
            .seek_relative(number_size as i64)
            .map_err(|_| DecodeError::UnexpectedEOF)?;

        Ok(AMF0Value::Number(number))
    }

    fn decode_bool(&mut self) -> Result<AMF0Value<'a>, DecodeError> {
        let value = self.get_buf()?.first().ok_or(DecodeError::InvalidBool)?;
        self.cursor
            .seek_relative(1)
            .map_err(|_| DecodeError::UnexpectedEOF)?;

        Ok(AMF0Value::Boolean(*value == 0x01))
    }

    pub fn decode_string(&mut self) -> Result<AMF0Value<'a>, DecodeError> {
        let length = u16::from_be_bytes(
            self.get_buf()?
                .get(..2)
                .ok_or(DecodeError::UnexpectedEOF)?
                .try_into()
                .map_err(|_| DecodeError::UnexpectedEOF)?,
        );
        self.cursor
            .seek_relative(2)
            .map_err(|_| DecodeError::UnexpectedEOF)?;

        let value = self
            .get_buf()?
            .get(..length as usize)
            .ok_or(DecodeError::UnexpectedEOF)?;

        self.cursor
            .seek_relative(length as i64)
            .map_err(|_| DecodeError::UnexpectedEOF)?;

        Ok(AMF0Value::String(str::from_utf8(value)?))
    }

    fn read_kv_pairs(&mut self) -> Result<HashMap<&'a str, AMF0Value<'a>>, DecodeError> {
        let mut map = HashMap::new();
        while self.get_buf()?.get(..3) != Some(&amf0_type_marker::OBJECT_END_MARKER) {
            let AMF0Value::String(key) = self.decode_string()? else {
                return Err(DecodeError::InvalidObjectKey);
            };
            let value = self.decode()?;
            map.insert(key, value);
        }
        self.cursor
            .seek_relative(3)
            .map_err(|_| DecodeError::UnexpectedEOF)?;

        Ok(map)
    }

    fn decode_object(&mut self) -> Result<AMF0Value<'a>, DecodeError> {
        Ok(AMF0Value::Object(self.read_kv_pairs()?))
    }

    #[cfg(test)]
    fn position(&self) -> u64 {
        self.cursor.position()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_string() {
        let actual = "hello world";
        let bytes = [
            (actual.len() as u16).to_be_bytes().as_slice(),
            actual.as_bytes(),
        ]
        .concat();
        let mut decoder = Decoder::new(&bytes);
        assert_eq!(decoder.decode_string(), Ok(AMF0Value::String(actual)));
        assert_eq!(decoder.position(), bytes.len() as u64);
    }

    #[test]
    fn test_decode_number() {
        let actual: f64 = rand::random();
        let bytes = actual.to_be_bytes();
        let mut decoder = Decoder::new(&bytes);
        assert_eq!(decoder.decode_number(), Ok(AMF0Value::Number(actual)));
        assert_eq!(decoder.position(), 8);
    }

    #[test]
    fn test_decode_bool() {
        let mut decoder = Decoder::new(&[1]);
        assert_eq!(decoder.decode_bool(), Ok(AMF0Value::Boolean(true)));
        assert_eq!(decoder.position(), 1);
        let mut decoder = Decoder::new(&[0]);
        assert_eq!(decoder.decode_bool(), Ok(AMF0Value::Boolean(false)));
        assert_eq!(decoder.position(), 1);
    }

    #[test]
    fn test_decode_string_with_marker() {
        let actual = "hello world";
        let bytes = [
            &[amf0_type_marker::STRING],
            (actual.len() as u16).to_be_bytes().as_slice(),
            actual.as_bytes(),
        ]
        .concat();

        let mut decoder = Decoder::new(bytes.as_slice());
        assert_eq!(decoder.decode(), Ok(AMF0Value::String(actual)));
        assert_eq!(decoder.position(), bytes.len() as u64);
    }

    #[test]
    fn test_decode_number_with_marker() {
        let actual: f64 = rand::random();
        let bytes = [&[amf0_type_marker::NUMBER], actual.to_be_bytes().as_slice()].concat();
        let mut decoder = Decoder::new(bytes.as_slice());
        assert_eq!(decoder.decode(), Ok(AMF0Value::Number(actual)));
        assert_eq!(decoder.position(), bytes.len() as u64);
    }

    #[test]
    fn test_decode_bool_with_marker() {
        let mut decoder = Decoder::new(&[amf0_type_marker::BOOL, 0x01]);
        assert_eq!(decoder.decode(), Ok(AMF0Value::Boolean(true)));
        assert_eq!(decoder.position(), 2);
        let mut decoder = Decoder::new(&[amf0_type_marker::BOOL, 0x00]);
        assert_eq!(decoder.decode(), Ok(AMF0Value::Boolean(false)));
        assert_eq!(decoder.position(), 2);
    }

    #[test]
    fn test_encode_decode_number() {
        let val = AMF0Value::Number(rand::random());
        let bytes = val.serialize();
        let mut decoder = Decoder::new(&bytes);
        assert_eq!(Ok(val), decoder.decode());
        assert_eq!(decoder.position(), bytes.len().try_into().unwrap());
    }

    #[test]
    fn test_encode_decode_null() {
        let val = AMF0Value::Null;
        let bytes = val.serialize();
        let mut decoder = Decoder::new(&bytes);
        assert_eq!(Ok(val), decoder.decode());
        assert_eq!(decoder.position(), bytes.len().try_into().unwrap());
    }

    #[test]
    fn test_encode_decode_bool() {
        let val = AMF0Value::Boolean(false);
        let bytes = val.serialize();
        let mut decoder = Decoder::new(&bytes);
        assert_eq!(Ok(val), decoder.decode());
        assert_eq!(decoder.position(), bytes.len().try_into().unwrap());
    }

    #[test]
    fn test_encode_decode_string() {
        let val = AMF0Value::String("hello world");
        let bytes = val.serialize();
        let mut decoder = Decoder::new(&bytes);
        assert_eq!(Ok(val), decoder.decode());
        assert_eq!(decoder.position(), bytes.len().try_into().unwrap());
    }

    #[test]
    fn test_encode_decode_object() {
        let val = AMF0Value::Object(HashMap::from([
            ("test", AMF0Value::Number(rand::random())),
            ("hello", AMF0Value::String("world")),
            ("test3", AMF0Value::Boolean(true)),
        ]));
        let bytes = val.serialize();
        let mut decoder = Decoder::new(&bytes);
        assert_eq!(Ok(val), decoder.decode());
        assert_eq!(decoder.position(), bytes.len().try_into().unwrap());
    }

    #[test]
    fn test_encode_decode_ecma_array() {
        let val = AMF0Value::EcmaArray(HashMap::from([
            ("test", AMF0Value::Number(rand::random())),
            ("hello", AMF0Value::String("world")),
            ("test3", AMF0Value::Boolean(true)),
        ]));
        let bytes = val.serialize();
        let mut decoder = Decoder::new(&bytes);
        assert_eq!(Ok(val), decoder.decode());
        assert_eq!(decoder.position(), bytes.len().try_into().unwrap());
    }
}
