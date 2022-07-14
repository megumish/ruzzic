use byteorder::{NetworkEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::{FromReadBytesWith, ReadBytesTo, ReadBytesToWith};

use crate::{
    extension::{Extension, Extensions},
    CipherSuite, LegacyVersion,
};

use super::HandshakeType;

#[derive(Debug)]
pub struct Body {
    legacy_version: LegacyVersion,
    random: [u8; 32],
    legacy_session_id: Vec<u8>,
    cipher_suites: Vec<CipherSuite>,
    legacy_compression_methods: Vec<u8>,
    extensions: Extensions,
}

impl FromReadBytesWith<HandshakeType> for Body {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
        handshake_type: HandshakeType,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let _remain_length = input.read_uint::<NetworkEndian>(3)? as usize;
        let legacy_version = input.read_bytes_to()?;
        let random = {
            let mut buf = [0; 32];
            input.read_exact(&mut buf)?;
            buf
        };
        let legacy_session_id = {
            let length = input.read_u8()?;
            let mut buf = vec![0; length as usize];
            input.read_exact(&mut buf)?;
            buf
        };
        let cipher_suites = {
            let length = input.read_u16::<NetworkEndian>()? as usize;
            let mut cipher_suites = Vec::new();
            for i in 0..length / CipherSuite::size_of() {
                let cipher_suite = input.read_bytes_to()?;
                cipher_suites.push(cipher_suite);
            }
            cipher_suites
        };
        let legacy_compression_methods = {
            let length = input.read_u8()? as usize;
            let mut buf = vec![0; length];
            input.read_exact(&mut buf)?;
            buf
        };
        let extensions = {
            let length = input.read_u16::<NetworkEndian>()? as usize;
            let mut total_length = 0;
            let mut extensions = Vec::new();
            while total_length < length {
                let e: Extension = input.read_bytes_to_with(handshake_type.clone())?;
                total_length += e.size_of();
                extensions.push(e);
            }
            extensions
        };
        Ok(Self {
            legacy_version,
            random,
            legacy_session_id,
            cipher_suites,
            legacy_compression_methods,
            extensions,
        })
    }
}
