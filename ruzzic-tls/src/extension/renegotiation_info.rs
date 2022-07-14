use byteorder::{NetworkEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::FromReadBytesWith;

#[derive(Debug)]
pub struct Body {
    renegotiated_connection: Vec<u8>,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let length = input.read_u16::<NetworkEndian>()?;
        let mut renegotiated_connection = vec![0u8; length as usize];
        input.read_exact(&mut renegotiated_connection);
        Ok(Self {
            renegotiated_connection,
        })
    }
}

impl Body {
    pub(crate) fn size_of(&self) -> usize {
        1 + self.renegotiated_connection.len()
    }
}
