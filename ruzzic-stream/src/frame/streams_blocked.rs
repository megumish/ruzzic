use crate::{read_varint, stream::StreamDirection, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    direction: StreamDirection,
    maximum_streams: VarInt,
}

impl Body {
    pub fn read_bytes_to(
        input: &mut impl std::io::Read,
        frame_type: u64,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let direction = if frame_type == 0x16 {
            StreamDirection::Bidirectional
        } else if frame_type == 0x17 {
            StreamDirection::Unidirectional
        } else {
            unreachable!("Invalid frame type")
        };
        let maximum_streams = read_varint(input)?;
        Ok(Self {
            direction,
            maximum_streams,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn streams_blocked() {
        let buf = [0];
        let mut input = Cursor::new(buf);
        let actual: Body = Body::read_bytes_to(&mut input, 0x16).unwrap();
        let expected = Body {
            direction: StreamDirection::Bidirectional,
            maximum_streams: VarInt(0),
        };
        assert_eq!(actual, expected);
    }
}
