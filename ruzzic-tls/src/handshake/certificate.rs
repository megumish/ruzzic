use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::CertificateEntry;

#[derive(Debug, PartialEq)]
pub struct Body {
    certificate_request_context: Vec<u8>,
    certificate_list: Vec<CertificateEntry>,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
