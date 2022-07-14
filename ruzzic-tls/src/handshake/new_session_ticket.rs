use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::Extensions;

#[derive(Debug)]
pub struct Body {
    ticket_lifetime: u32,
    ticket_age_add: u32,
    ticket_nonce: Vec<u8>,
    ticket: Vec<u8>,
    extensions: Extensions,
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
