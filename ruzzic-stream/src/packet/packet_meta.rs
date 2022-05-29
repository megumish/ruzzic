use bitvec::prelude::*;
use ruzzic_common::read_bytes_to::{FromReadBytesWith, ReadBytesTo};

use crate::Version;

use super::{long_header, PacketBodyType};

#[derive(Debug, Clone, PartialEq)]
pub struct PacketMeta {
    pub(crate) first_byte: FirstByte,
    pub(crate) version: Version,
}

impl FromReadBytesWith<()> for PacketMeta {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let first_byte = input.read_bytes_to()?;
        let version = input.read_bytes_to()?;
        Ok(Self {
            first_byte,
            version,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FirstByte(pub(crate) BitArr!(for 8, in Msb0, u8));

impl FromReadBytesWith<()> for FirstByte {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
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

    pub fn long_packet_type(&self) -> long_header::PacketType {
        match self.0[2..4].load::<u8>() {
            0 => long_header::PacketType::Initial,
            1 => long_header::PacketType::ZeroRTT,
            2 => long_header::PacketType::Handshake,
            3 => long_header::PacketType::Retry,
            _ => unreachable!("this must be 2bit value"),
        }
    }

    fn get_type(&self) -> PacketBodyType {
        if self.is_long() {
            PacketBodyType::Long
        } else {
            PacketBodyType::Short
        }
    }

    fn packet_number_length(&self) -> u8 {
        self.0[6..8].load_be::<u8>() + 1
    }

    fn raw_length(&self) -> usize {
        1
    }
}

impl PacketMeta {
    pub(crate) fn get_type(&self) -> PacketBodyType {
        self.first_byte.get_type()
    }

    pub(crate) fn long_packet_type(&self) -> long_header::PacketType {
        self.first_byte.long_packet_type()
    }

    pub(crate) fn version(&self) -> Version {
        self.version
    }

    pub(crate) fn packet_number_length(&self) -> u8 {
        self.first_byte.packet_number_length()
    }

    pub(crate) fn raw_length(&self) -> usize {
        self.first_byte.raw_length() + self.version.raw_length()
    }
}
