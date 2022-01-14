use crate::{read_varint, FromReadBytes, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    offset: VarInt,
    data: CryptoData,
}

#[derive(Debug, PartialEq)]
pub struct CryptoData(Vec<u8>);

impl FromReadBytes for Body {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let offset = read_varint(input)?;
        let length = read_varint(input)?;
        let mut buf = vec![0; length.to_u64() as usize];
        input.read_exact(&mut buf)?;
        Ok(Self {
            offset,
            data: CryptoData(buf),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::ReadBytesTo;

    #[test]
    fn crypto() {
        let buf = vec![0, 1, 0];
        let mut input = Cursor::new(buf);
        let actual: Body = input.read_bytes_to().unwrap();
        let expected = Body {
            offset: VarInt(0),
            data: CryptoData(vec![0]),
        };
        assert_eq!(actual, expected);
    }
}
