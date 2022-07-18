use byteorder::{NetworkEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::FromReadBytesWith;

#[derive(Debug, PartialEq)]
pub struct Body {
    cookie: Vec<u8>,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let length = input.read_u16::<NetworkEndian>()?;
        let cookie = {
            let mut cookie = vec![0u8; length as usize];
            input.read_exact(&mut cookie)?;
            cookie
        };
        Ok(Body { cookie })
    }
}

impl Body {
    pub(crate) fn size_of(&self) -> usize {
        2 + self.cookie.len()
    }
}
