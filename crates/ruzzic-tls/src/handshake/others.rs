use ruzzic_common::read_bytes_to::FromReadBytesWith;

#[derive(Debug, PartialEq)]
pub struct Body;

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
