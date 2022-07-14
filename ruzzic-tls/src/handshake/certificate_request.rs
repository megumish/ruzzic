use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::extension::Extensions;

use super::HandshakeType;

#[derive(Debug)]
pub struct Body {
    certificate_request_context: Vec<u8>,
    extensions: Extensions,
}

impl FromReadBytesWith<HandshakeType> for Body {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
        _: HandshakeType,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
