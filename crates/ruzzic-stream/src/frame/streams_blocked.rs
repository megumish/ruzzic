use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::{read_varint, stream::StreamDirection, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    direction: StreamDirection,
    maximum_streams: VarInt,
}

impl FromReadBytesWith<u64> for Body {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
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
    use ruzzic_common::read_bytes_to::ReadBytesToWith;
    use std::io::Cursor;

    use super::*;

    #[test]
    fn streams_blocked() {
        let buf = [0];
        let mut input = Cursor::new(buf);
        let actual: Body = input.read_bytes_to_with(0x16).unwrap();
        let expected = Body {
            direction: StreamDirection::Bidirectional,
            maximum_streams: VarInt(0),
        };
        assert_eq!(actual, expected);
    }
}
