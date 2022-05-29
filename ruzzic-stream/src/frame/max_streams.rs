use std::io::Read;

use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::{read_varint, stream::StreamDirection, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    kind: StreamDirection,
    maximum_streams: VarInt,
}

impl FromReadBytesWith<u64> for Body {
    fn from_read_bytes_with<R: Read>(input: &mut R, frame_type: u64) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
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
    use ruzzic_common::read_bytes_to::ReadBytesToWith;

    use super::*;
    use std::io::Cursor;

    #[test]
    fn max_streams() {
        let buf = [0];
        let mut input = Cursor::new(buf);
        let actual: Body = input.read_bytes_to_with(0x12).unwrap();
        let expected = Body {
            kind: StreamDirection::Bidirectional,
            maximum_streams: VarInt(0),
        };
        assert_eq!(actual, expected);
    }
}
