use bitvec::prelude::*;

use crate::{read_bytes_to::FromReadBytesWith, Version};

use super::PacketType;

#[derive(Debug, PartialEq)]
pub struct PacketMeta {
    first_byte: FirstByte,
    version: Version,
}

#[derive(Debug, PartialEq)]
pub struct FirstByte(BitArr!(for 8, in Msb0, u8));

impl FromReadBytesWith<()> for FirstByte {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
        with: (),
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut buf = [0; 1];
        input.read_exact(&mut buf)?;
        let mut first_byte = bitarr![Msb0, u8; 0; 8];
        first_byte.store(buf[0]);
        Ok(Self(first_byte))
    }
}

impl FirstByte {
    pub fn is_long(&self) -> bool {
        self.0[0]
    }

    pub fn is_short(&self) -> bool {
        !self.is_long()
    }

    pub fn is_valid(&self) -> bool {
        self.0[1]
    }

    pub fn long_packet_type(&self) -> PacketType {
        match self.0[2..4].load::<u8>() {
            0 => PacketType::Initial,
            1 => PacketType::ZeroRTT,
            2 => PacketType::Handshake,
            3 => PacketType::Retry,
            _ => unreachable!("this must be 2bit value"),
        }
    }
}
