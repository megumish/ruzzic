use crate::{read_varint, FromReadBytes, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    sequence_number: VarInt,
}

impl FromReadBytes for Body {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let sequence_number = read_varint(input)?;
        Ok(Self { sequence_number })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReadBytesTo;

    use std::io::Cursor;

    #[test]
    fn retire_connection_id() {
        let buf = [0];
        let mut input = Cursor::new(buf);
        let actual: Body = input.read_bytes_to().unwrap();
        let expected = Body {
            sequence_number: VarInt(0),
        };
        assert_eq!(actual, expected);
    }
}
