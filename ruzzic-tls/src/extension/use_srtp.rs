use ruzzic_common::read_bytes_to::FromReadBytesWith;

#[derive(Debug)]
pub struct Body {
    SrtpProtectionProfiles: Vec<[u8; 2]>,
    SrtpMki: Vec<u8>,
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
