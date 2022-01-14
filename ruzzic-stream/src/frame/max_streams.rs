use std::io::Read;

use crate::{read_varint, stream::StreamDirection, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    kind: StreamDirection,
    maximum_streams: VarInt,
}

impl Body {
    pub fn read_bytes_to(input: &mut impl Read, frame_type: u64) -> Result<Self, std::io::Error> {
        let kind = if frame_type == 0x12 {
            StreamDirection::Bidirectional
        } else if frame_type == 0x13 {
            StreamDirection::Unidirectional
        } else {
            unreachable!("Invalid frame type")
        };
        let maximum_streams = read_varint(input)?;
        Ok(Self {
            kind,
            maximum_streams,
        })
    }
}
#[cfg(test)]
mod tests {
    use crate::{stream::StreamDirection, VarInt};

    use super::Body;
    use std::io::Cursor;

    #[test]
    fn max_streams() {
        let buf = [0];
        let mut input = Cursor::new(buf);
        let actual: Body = Body::read_bytes_to(&mut input, 0x12).unwrap();
        let expected = Body {
            kind: StreamDirection::Bidirectional,
            maximum_streams: VarInt(0),
        };
        assert_eq!(actual, expected);
    }
}
