use crate::{read_varint, FromReadBytes, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    maximum_data: VarInt,
}

impl FromReadBytes for Body {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let maximum_data = read_varint(input)?;
        Ok(Self { maximum_data })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::Body;
    use crate::{ReadBytesTo, VarInt};

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
