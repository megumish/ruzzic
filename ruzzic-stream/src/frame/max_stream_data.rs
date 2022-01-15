use crate::{read_bytes_to::FromReadBytesWith, read_varint, stream::StreamID, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    stream_id: StreamID,
    maximum_stream_data: VarInt,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let stream_id = read_varint(input)?;
        let maximum_stream_data = read_varint(input)?;
        Ok(Self {
            stream_id: StreamID(stream_id.to_u64()),
            maximum_stream_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::Body;
    use crate::{read_bytes_to::ReadBytesTo, stream::StreamID, VarInt};

    #[test]
    fn max_stream_data() {
        let buf = [0, 0];
        let mut input = Cursor::new(buf);
        let actual: Body = input.read_bytes_to().unwrap();
        let expected = Body {
            stream_id: StreamID(0),
            maximum_stream_data: VarInt(0),
        };
        assert_eq!(actual, expected);
    }
}
