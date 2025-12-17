use crate::{
    amf::{AMF0Value, DecodeError, Decoder},
    messages,
};

#[derive(Debug, PartialEq)]
pub enum NetStreamCommand<'a> {
    Play {
        stream_name: &'a str,
        start: Option<f64>,
        duration: Option<f64>,
        reset: Option<bool>,
    },
    Play2 {
        parameters: AMF0Value<'a>,
    },
    DeleteStream {
        stream_id: u32,
    },
    CloseStream {
        stream_id: u32,
    },
    ReceiveAudio {
        should_receive: bool,
    },
    ReceiveVideo {
        should_receive: bool,
    },
    Publish {
        /// usually used as stream key
        publishing_name: &'a str,
        publishing_type: &'a str,
    },
    Seek {
        milliseconds: f64,
    },
    Pause {
        is_paused: bool,
        milliseconds: f64,
    },
}

impl<'a> NetStreamCommand<'a> {
    fn parse_optional(decoder: &mut Decoder<'a>) -> Result<Option<AMF0Value<'a>>, DecodeError> {
        decoder.decode().map(Some).or_else(|err| {
            if matches!(err, DecodeError::UnexpectedEOF) {
                Ok(None)
            } else {
                Err(err)
            }
        })
    }

    fn parse_play(buf: &'a [u8]) -> Result<Self, messages::command::ParseError> {
        let mut decoder = Decoder::new(buf);
        let stream_name = decoder.decode()?.try_into()?;
        let start = Self::parse_optional(&mut decoder)?
            .map(|val| val.try_into())
            .transpose()?;
        let duration = Self::parse_optional(&mut decoder)?
            .map(|val| val.try_into())
            .transpose()?;
        let reset = Self::parse_optional(&mut decoder)?
            .map(|val| val.try_into())
            .transpose()?;
        Ok(Self::Play {
            stream_name,
            start,
            duration,
            reset,
        })
    }

    fn parse_play2(buf: &'a [u8]) -> Result<Self, messages::command::ParseError> {
        let mut decoder = Decoder::new(buf);
        Ok(Self::Play2 {
            parameters: decoder.decode()?,
        })
    }

    fn parse_delete_stream(buf: &'a [u8]) -> Result<Self, messages::command::ParseError> {
        let mut decoder = Decoder::new(buf);
        let stream_id: f64 = decoder.decode()?.try_into()?;
        Ok(Self::DeleteStream {
            stream_id: stream_id as u32,
        })
    }

    fn parse_close_stream(buf: &'a [u8]) -> Result<Self, messages::command::ParseError> {
        let mut decoder = Decoder::new(buf);
        let stream_id: f64 = decoder.decode()?.try_into()?;
        Ok(Self::CloseStream {
            stream_id: stream_id as u32,
        })
    }

    fn parse_receive_audio(buf: &'a [u8]) -> Result<Self, messages::command::ParseError> {
        Ok(Self::ReceiveAudio {
            should_receive: Decoder::new(buf).decode()?.try_into()?,
        })
    }

    fn parse_receive_video(buf: &'a [u8]) -> Result<Self, messages::command::ParseError> {
        Ok(Self::ReceiveVideo {
            should_receive: Decoder::new(buf).decode()?.try_into()?,
        })
    }

    fn parse_publish(buf: &'a [u8]) -> Result<Self, messages::command::ParseError> {
        let mut decoder = Decoder::new(buf);
        let publishing_name = decoder.decode()?.try_into()?;
        let publishing_type = decoder.decode()?.try_into()?;

        Ok(Self::Publish {
            publishing_name,
            publishing_type,
        })
    }

    fn parse_seek(buf: &'a [u8]) -> Result<Self, messages::command::ParseError> {
        Ok(Self::Seek {
            milliseconds: Decoder::new(buf).decode()?.try_into()?,
        })
    }

    fn parse_pause(buf: &'a [u8]) -> Result<Self, messages::command::ParseError> {
        let mut decoder = Decoder::new(buf);
        let is_paused = decoder.decode()?.try_into()?;
        let milliseconds = decoder.decode()?.try_into()?;
        Ok(Self::Pause {
            is_paused,
            milliseconds,
        })
    }

    pub fn parse(command: &'a str, buf: &'a [u8]) -> Result<Self, messages::command::ParseError> {
        Ok(match command {
            "play" => Self::parse_play(buf)?,
            "play2" => Self::parse_play2(buf)?,
            "deleteStream" => Self::parse_delete_stream(buf)?,
            "closeStream" => Self::parse_close_stream(buf)?,
            "receiveAudio" => Self::parse_receive_audio(buf)?,
            "receiveVideo" => Self::parse_receive_video(buf)?,
            "publish" => Self::parse_publish(buf)?,
            "seek" => Self::parse_seek(buf)?,
            "pause" => Self::parse_pause(buf)?,
            value => {
                return Err(messages::command::ParseError::InvalidCommand(
                    value.to_owned(),
                ));
            }
        })
    }

    pub fn name(&self) -> &str {
        match self {
            NetStreamCommand::Play { .. } => "play",
            NetStreamCommand::Play2 { .. } => "play2",
            NetStreamCommand::DeleteStream { .. } => "deleteStream",
            NetStreamCommand::CloseStream { .. } => "closeStream",
            NetStreamCommand::ReceiveAudio { .. } => "receiveAudio",
            NetStreamCommand::ReceiveVideo { .. } => "receiveVideo",
            NetStreamCommand::Publish { .. } => "publish",
            NetStreamCommand::Seek { .. } => "seek",
            NetStreamCommand::Pause { .. } => "pause",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::amf::AMF0Value;

    use super::*;

    #[test]
    fn test_parse_play() {
        let start = AMF0Value::Number(rand::random());
        let duration = AMF0Value::Number(rand::random());
        let buf = [
            AMF0Value::String("test").serialize(),
            start.serialize(),
            duration.serialize(),
            AMF0Value::Boolean(true).serialize(),
        ]
        .concat();
        let actual = NetStreamCommand::parse("play", &buf).unwrap();
        assert_eq!(
            actual,
            NetStreamCommand::Play {
                stream_name: "test",
                start: Some(start.try_into().unwrap()),
                duration: Some(duration.try_into().unwrap()),
                reset: Some(true)
            }
        )
    }

    #[test]
    fn test_parse_play2() {
        let obj = AMF0Value::Object(HashMap::from([
            ("stream_name", AMF0Value::String("test")),
            ("start", AMF0Value::Number(rand::random())),
        ]));
        let bytes = obj.serialize();
        let actual = NetStreamCommand::parse("play2", &bytes).unwrap();
        assert_eq!(actual, NetStreamCommand::Play2 { parameters: obj })
    }

    #[test]
    fn test_parse_delete_stream() {
        let expected = rand::random();
        let val = AMF0Value::Number(expected);
        let bytes = val.serialize();
        let actual = NetStreamCommand::parse("deleteStream", &bytes).unwrap();
        assert_eq!(
            actual,
            NetStreamCommand::DeleteStream {
                stream_id: expected as u32
            }
        )
    }

    #[test]
    fn test_parse_close_stream() {
        let expected = rand::random();
        let val = AMF0Value::Number(expected);
        let bytes = val.serialize();
        let actual = NetStreamCommand::parse("closeStream", &bytes).unwrap();
        assert_eq!(
            actual,
            NetStreamCommand::CloseStream {
                stream_id: expected as u32
            }
        )
    }

    #[test]
    fn test_parse_receive_audio() {
        let expected = rand::random();
        let val = AMF0Value::Boolean(expected);
        let bytes = val.serialize();
        let actual = NetStreamCommand::parse("receiveAudio", &bytes).unwrap();
        assert_eq!(
            actual,
            NetStreamCommand::ReceiveAudio {
                should_receive: expected
            }
        )
    }

    #[test]
    fn test_parse_receive_video() {
        let expected = rand::random();
        let val = AMF0Value::Boolean(expected);
        let bytes = val.serialize();
        let actual = NetStreamCommand::parse("receiveVideo", &bytes).unwrap();
        assert_eq!(
            actual,
            NetStreamCommand::ReceiveVideo {
                should_receive: expected
            }
        )
    }

    #[test]
    fn test_parse_publish() {
        let publishing_name = AMF0Value::String("stream_key");
        let publishing_type = AMF0Value::String("live");
        let bytes = [publishing_name.serialize(), publishing_type.serialize()].concat();
        let actual = NetStreamCommand::parse("publish", &bytes).unwrap();
        assert_eq!(
            actual,
            NetStreamCommand::Publish {
                publishing_name: publishing_name.try_into().unwrap(),
                publishing_type: publishing_type.try_into().unwrap()
            }
        )
    }

    #[test]
    fn test_parse_seek() {
        let val = AMF0Value::Number(rand::random());
        let bytes = val.serialize();
        let actual = NetStreamCommand::parse("seek", &bytes).unwrap();
        assert_eq!(
            actual,
            NetStreamCommand::Seek {
                milliseconds: val.try_into().unwrap()
            }
        )
    }

    #[test]
    fn test_parse_pause() {
        let is_paused = AMF0Value::Boolean(rand::random());
        let milliseconds = AMF0Value::Number(rand::random());
        let bytes = [is_paused.serialize(), milliseconds.serialize()].concat();
        let actual = NetStreamCommand::parse("pause", &bytes).unwrap();
        assert_eq!(
            actual,
            NetStreamCommand::Pause {
                is_paused: is_paused.try_into().unwrap(),
                milliseconds: milliseconds.try_into().unwrap()
            }
        )
    }

    #[test]
    fn test_parse_play_command_from_amf() {
        let expected = NetStreamCommand::Play {
            stream_name: "test",
            start: None,
            duration: None,
            reset: None,
        };
        let bytes = [AMF0Value::String("test")]
            .map(|val| val.serialize())
            .concat();

        let actual = NetStreamCommand::parse_play(&bytes).unwrap();
        assert_eq!(actual, expected);
    }
}
