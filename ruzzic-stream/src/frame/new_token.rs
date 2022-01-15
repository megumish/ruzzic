use crate::{read_bytes_to::FromReadBytesWith, read_varint, Token};

#[derive(Debug, PartialEq)]
pub struct Body {
    // The token MUST NOT be empty
    // TODO: so implment error handling
    token: Token,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let length = read_varint(input)?;
        let mut buf = vec![0; length.to_u64() as usize];
        input.read_exact(&mut buf)?;
        Ok(Self { token: Token(buf) })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::read_bytes_to::ReadBytesTo;

    #[test]
    fn new_token() {
        let buf = vec![1, 0];
        let mut input = Cursor::new(buf);
        let actual: Body = input.read_bytes_to().unwrap();
        let expected = Body {
            token: Token(vec![0]),
        };
        assert_eq!(actual, expected);
    }
}
