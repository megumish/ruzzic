use ruzzic_common::read_bytes_to::FromReadBytesWith;

#[derive(Debug)]
pub struct Body {
    key_update_request: KeyUpdateRequest,
}

#[derive(Debug)]
pub enum KeyUpdateRequest {
    UpdateNotRequested,
    UpdateRequested,
    Others(u8),
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
