use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::{read_varint, stream::StreamID, ApplicationProtocolErrorCode, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    stream_id: StreamID,
    error_code: ApplicationProtocolErrorCode,
    final_size: VarInt,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let stream_id = read_varint(input)?;
        let error_code = read_varint(input)?;
        let final_size = read_varint(input)?;
        Ok(Self {
            stream_id: StreamID(stream_id.to_u64()),
            error_code: ApplicationProtocolErrorCode(error_code.to_u64()),
            final_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use ruzzic_common::read_bytes_to::ReadBytesTo;
    use std::io::Cursor;

    use super::*;

    #[test]
    fn reset_stream() {
        let buf = vec![0, 0, 0];
        let mut input = Cursor::new(buf);
        let actual: Body = input.read_bytes_to().unwrap();
        let expected = Body {
            stream_id: StreamID(0),
            error_code: ApplicationProtocolErrorCode(0),
            final_size: VarInt(0),
        };
        assert_eq!(actual, expected);
    }
}
