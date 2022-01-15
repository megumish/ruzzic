use crate::read_bytes_to::FromReadBytesWith;

#[derive(Debug, PartialEq)]
pub struct ConnectionID(pub(crate) Vec<u8>);

impl FromReadBytesWith<()> for ConnectionID {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
