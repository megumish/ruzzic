use byteorder::{NetworkEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::FromReadBytesWith;

use crate::handshake::HandshakeType;

#[derive(Debug)]
pub enum Body {
    NewSessionTicket { max_early_data_size: u32 },
    ClientHello,
    EncryptedExtension,
    Others(HandshakeType),
}

impl FromReadBytesWith<HandshakeType> for Body {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
        handshake_type: HandshakeType,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(match handshake_type {
            HandshakeType::NewSessionTicket => {
                let max_early_data_size = input.read_u32::<NetworkEndian>()?;
                Body::NewSessionTicket {
                    max_early_data_size,
                }
            }
            HandshakeType::ClientHello => Body::ClientHello,
            HandshakeType::EncryptedExtensions => Body::EncryptedExtension,
            x => Body::Others(x),
        })
    }
}

impl Body {
    pub(crate) fn size_of(&self) -> usize {
        match self {
            Body::NewSessionTicket {
                max_early_data_size: _,
            } => 4,
            Body::ClientHello => 0,
            Body::EncryptedExtension => 0,
            Body::Others(_) => 0,
        }
    }
}
