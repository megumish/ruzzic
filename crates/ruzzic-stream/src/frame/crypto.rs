use std::io::Cursor;

use ruzzic_common::read_bytes_to::{FromReadBytesWith, ReadBytesTo};
use ruzzic_tls::handshake::Handshake;

use crate::{read_varint, VarInt};

#[derive(Debug, PartialEq)]
pub struct Body {
    offset: VarInt,
    tls_handshake: Handshake,
}

#[derive(Debug, PartialEq)]
pub struct CryptoData(Vec<u8>);

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let offset = read_varint(input)?;
        let length = read_varint(input)?;
        let mut input = {
            let mut buf = vec![0; length.to_u64() as usize];
            input.read_exact(&mut buf)?;
            Cursor::new(buf)
        };

        Ok(Self {
            offset,
            tls_handshake: input.read_bytes_to()?,
        })
    }
}
