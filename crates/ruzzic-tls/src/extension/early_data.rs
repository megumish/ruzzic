use std::io::Cursor;

use byteorder::{NetworkEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::{FromReadBytesWith, ReadBytesToWith};

use crate::handshake::HandshakeType;

#[derive(Debug, PartialEq)]
pub struct Body {
    length: usize,
    value: EarlyData,
}

#[derive(Debug, PartialEq)]
pub enum EarlyData {
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
        let length = input.read_u16::<NetworkEndian>()? as usize;
        let mut input = {
            let mut buf = vec![0u8; length as usize];
            input.read_exact(&mut buf)?;
            Cursor::new(buf)
        };
        let value = input.read_bytes_to_with(handshake_type)?;
        Ok(Self { length, value })
    }
}

impl FromReadBytesWith<HandshakeType> for EarlyData {
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
                EarlyData::NewSessionTicket {
                    max_early_data_size,
                }
            }
            HandshakeType::ClientHello => EarlyData::ClientHello,
            HandshakeType::EncryptedExtensions => EarlyData::EncryptedExtension,
            x => EarlyData::Others(x),
        })
    }
}

impl Body {
    pub(crate) fn size_of(&self) -> usize {
        2 + self.length
    }
}
