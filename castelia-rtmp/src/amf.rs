// A naive amf0 parser
// implemented the bare minimum to parse amf0 for the rtmp protocol
// seems like not the full specification/all the types are used in the protocol

use std::{
    collections::HashMap,
    io::{Cursor, Seek},
    str,
};

use thiserror::Error;

mod amf0_type_marker {
    pub const NUMBER: u8 = 0x00;
    pub const BOOL: u8 = 0x01;
    pub const STRING: u8 = 0x02;
    pub const OBJECT_START: u8 = 0x03;

    // needs to be preceeded by 2 0x00s
    // so actual object end is 0x00, 0x00, 0x09
    pub const OBJECT_END: u8 = 0x09;
    pub const NULL: u8 = 0x05;
}

#[derive(Debug, PartialEq)]
pub enum AMF0Value<'a> {
    Number(f64),
    Boolean(bool),
    String(&'a str),
    Object(HashMap<&'a str, AMF0Value<'a>>),
    Null,
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
    #[error("Missing type marker")]
    MissingTypeMarker,
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

    fn get_buf(&self) -> Result<&'a [u8], DecodeError> {
        self.cursor
            .get_ref()
            .get(self.cursor.position() as usize..)
            .ok_or(DecodeError::UnexpectedEOF)
    }

    pub fn decode(&mut self) -> Result<AMF0Value<'a>, DecodeError> {
        let type_marker = self
            .get_buf()?
            .first()
            .ok_or(DecodeError::MissingTypeMarker)?;
        self.cursor
            .seek_relative(1)
            .map_err(|_| DecodeError::UnexpectedEOF)?;
        let value = match *type_marker {
            amf0_type_marker::NUMBER => self.decode_number()?,
            amf0_type_marker::BOOL => self.decode_bool()?,
            amf0_type_marker::STRING => self.decode_string()?,
            amf0_type_marker::OBJECT_START => self.decode_object()?,
            amf0_type_marker::NULL => AMF0Value::Null,
            marker => return Err(DecodeError::UnknownMarker(marker)),
        };

        Ok(value)
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

        Ok(AMF0Value::String(str::from_utf8(value)?))
    }

    fn decode_object(&mut self) -> Result<AMF0Value<'a>, DecodeError> {
        let end_marker = [0x00, 0x00, amf0_type_marker::OBJECT_END];
        let mut obj = HashMap::new();
        while self.get_buf()?.get(..3) != Some(&end_marker) {
            let AMF0Value::String(key) = self.decode_string()? else {
                return Err(DecodeError::InvalidObjectKey);
            };
            let value = self.decode()?;
            obj.insert(key, value);
        }

        Ok(AMF0Value::Object(obj))
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
        assert_eq!(
            Decoder::new(&bytes).decode_string(),
            Ok(AMF0Value::String(actual))
        );
    }

    #[test]
    fn test_decode_number() {
        let actual: f64 = rand::random();
        assert_eq!(
            Decoder::new(&actual.to_be_bytes()).decode_number(),
            Ok(AMF0Value::Number(actual))
        );
    }

    #[test]
    fn test_decode_bool() {
        assert_eq!(
            Decoder::new(&[1]).decode_bool(),
            Ok(AMF0Value::Boolean(true))
        );
        assert_eq!(
            Decoder::new(&[0]).decode_bool(),
            Ok(AMF0Value::Boolean(false))
        );
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
        assert_eq!(
            Decoder::new(&bytes.as_slice()).decode(),
            Ok(AMF0Value::String(actual))
        );
    }
}
