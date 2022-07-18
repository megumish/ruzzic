use ruzzic_common::read_bytes_to::FromReadBytesWith;

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionID(pub(crate) Vec<u8>);

impl FromReadBytesWith<()> for ConnectionID {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}

impl ConnectionID {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}
