use std::io::{Cursor, Read};

use byteorder::{NetworkEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::FromReadBytesWith;

#[derive(Debug)]
pub struct Body {
    authorities: Vec<Authority>,
    total_length: usize,
}

#[derive(Debug)]
pub struct Authority {
    name: Vec<u8>,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let length = input.read_u16::<NetworkEndian>()? as usize;
        let mut input = {
            let mut buf = vec![0u8; length as usize];
            input.read_exact(&mut buf)?;
            Cursor::new(buf)
        };
        let mut authorities = Vec::new();
        let mut total_length = 0;
        while total_length < length {
            let name_length = input.read_u8()?;
            let mut name = vec![0; name_length as usize];
            input.read_exact(&mut name)?;
            authorities.push(Authority { name });
            total_length += name_length as usize + 1;
        }

        assert_eq!(total_length, length);
        Ok(Self {
            authorities,
            total_length,
        })
    }
}

impl Body {
    pub(crate) fn size_of(&self) -> usize {
        self.total_length
    }
}
