use crate::{read_varint, stream::StreamID, ApplicationProtocolErrorCode, FromReadBytes, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    stream_id: StreamID,
    error_code: ApplicationProtocolErrorCode,
    final_size: VarInt,
}

impl FromReadBytes for Body {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
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
    use std::io::Cursor;

    use super::*;
    use crate::ReadBytesTo;

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
