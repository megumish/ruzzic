use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::SignatureScheme;

#[derive(Debug, PartialEq)]
pub struct Body {
    algorithm: SignatureScheme,
    signature: Vec<u8>,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
