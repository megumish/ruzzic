use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::CertificateType;

#[derive(Debug)]
pub enum Body {
    Client(Vec<CertificateType>),
    Server(CertificateType),
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

impl Body {
    pub(crate) fn size_of(&self) -> usize {
        todo!()
    }
}
