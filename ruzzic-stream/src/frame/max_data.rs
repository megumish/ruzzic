use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::{read_varint, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    maximum_data: VarInt,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let maximum_data = read_varint(input)?;
        Ok(Self { maximum_data })
    }
}

#[cfg(test)]
mod tests {
    use ruzzic_common::read_bytes_to::ReadBytesTo;
    use std::io::Cursor;

    use super::*;

    #[test]
    fn max_data() {
        let buf = [0];
        let mut input = Cursor::new(buf);
        let actual: Body = input.read_bytes_to().unwrap();
        let expected = Body {
            maximum_data: VarInt(0),
        };
        assert_eq!(actual, expected);
    }
}
