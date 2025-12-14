use crate::{
    amf::{AMF0Value, Decoder},
    messages,
};

#[derive(Debug)]
pub enum NetStreamCommand<'a> {
    Play {
        stream_name: &'a str,
        start: f64,
        duration: f64,
        reset: bool,
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
    fn parse_play(buf: &'a [u8]) -> Result<Self, messages::command::ParseError> {
        let mut decoder = Decoder::new(buf);
        let stream_name = decoder.decode()?.try_into()?;
        let start = decoder.decode()?.try_into()?;
        let duration = decoder.decode()?.try_into()?;
        let reset = decoder.decode()?.try_into()?;
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
}
