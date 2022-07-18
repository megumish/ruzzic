use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::{CipherSuite, Extensions};

#[derive(Debug, PartialEq)]
pub struct Body {
    random: [u8; 32],
    legacy_session_id: Vec<u8>,
    cipher_suites: CipherSuite,
    legacy_compression_methods: u8,
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
