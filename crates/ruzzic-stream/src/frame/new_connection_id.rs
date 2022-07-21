use byteorder::{BigEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::{connection::ConnectionID, read_varint, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    sequence_number: VarInt,
    retire_prior_to: VarInt,
    connection_id: ConnectionID,
    stateless_reset_token: u128,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let sequence_number = read_varint(input)?;
        let retire_prior_to = read_varint(input)?;
        let length = input.read_u8()?;
        let mut connection_id = vec![0; length as usize];
        input.read_exact(&mut connection_id)?;
        let stateless_reset_token = input.read_u128::<BigEndian>()?;
        Ok(Self {
            sequence_number,
            retire_prior_to,
            connection_id: ConnectionID(connection_id),
            stateless_reset_token,
        })
    }
}

#[cfg(test)]
mod tests {
    use ruzzic_common::read_bytes_to::ReadBytesTo;
    use std::io::Cursor;

    use super::*;

    #[test]
    fn new_connection_id() {
        let buf = [
            0, // Sequence number
            0, // Retire Prior To
            1, // Connection ID Length
            1, // Connection ID
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Stateless Reset Token
        ];
        let mut input = Cursor::new(buf);
        let actual: Body = input.read_bytes_to().unwrap();
        let expected = Body {
            sequence_number: VarInt(0),
            retire_prior_to: VarInt(0),
            connection_id: ConnectionID(vec![1]),
            stateless_reset_token: 0,
        };
        assert_eq!(actual, expected);
    }
}
