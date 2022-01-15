use crate::{read_varint, VarInt};

use super::FrameType;

#[derive(Debug, PartialEq)]
pub struct Body {
    error_code: VarInt,
    frame_type: Option<FrameType>,
    reason_phrase: String,
}

impl Body {
    pub fn read_bytes_to(
        input: &mut impl std::io::Read,
        this_frame_type: u64,
    ) -> Result<Self, std::io::Error> {
        let error_code = read_varint(input)?;
        let frame_type = if this_frame_type == 0x1c {
            Some(FrameType::from_u64(read_varint(input)?.to_u64()))
        } else if this_frame_type == 0x1d {
            None
        } else {
            unreachable!("Invalid frame type")
        };
        let reason_phrase_length = read_varint(input)?.to_u64();
        let mut reason_phrase = vec![0; reason_phrase_length as usize];
        input.read_exact(&mut reason_phrase)?;
        // > This SHOULD be a UTF-8 encoded string [RFC3629], though the frame does not carry information, such as language tags, that would aid comprehension by any entity other than the one that created the text.
        let reason_phrase = unsafe { String::from_utf8_unchecked(reason_phrase) };
        Ok(Self {
            error_code,
            frame_type,
            reason_phrase,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn connection_close() {
        let buf = [0, 0, 1, 'a' as u8];
        let mut input = Cursor::new(buf);
        let actual: Body = Body::read_bytes_to(&mut input, 0x1c).unwrap();
        let expected = Body {
            error_code: VarInt(0),
            frame_type: Some(FrameType::Padding),
            reason_phrase: String::from("a"),
        };
        assert_eq!(actual, expected);
        eprintln!("{:?}", actual);
    }

    #[test]
    fn connection_close_without_frame_type() {
        let buf = [0, 1, 'a' as u8];
        let mut input = Cursor::new(buf);
        let actual: Body = Body::read_bytes_to(&mut input, 0x1d).unwrap();
        let expected = Body {
            error_code: VarInt(0),
            frame_type: None,
            reason_phrase: String::from("a"),
        };
        assert_eq!(actual, expected);
        eprintln!("{:?}", actual);
    }
}
