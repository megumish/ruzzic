use bitvec::prelude::*;
use byteorder::{BigEndian, ByteOrder};

use crate::FromReadBytes;

mod long_header;
pub enum Packet {
    Long(long_header::LongHeader),
}

#[derive(Debug, PartialEq)]
pub enum PacketType {
    Initial,
    ZeroRTT,
    Handshake,
    Retry,
}

#[derive(Debug, PartialEq)]
pub struct PacketNumber(pub(crate) u32);

impl PacketNumber {
    // TODO: check integer casting
    pub fn read_bytes_to(
        input: &mut impl std::io::Read,
        length: u16,
    ) -> Result<Self, std::io::Error> {
        let mut buf = vec![0u8; length as usize];
        input.read_exact(&mut buf)?;
        Ok(PacketNumber(
            BigEndian::read_uint(&buf, length as usize) as u32
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct PacketPayload(Vec<u8>);

impl FromReadBytes for PacketPayload {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut buf = Vec::new();
        input.read_to_end(&mut buf)?;
        Ok(PacketPayload(buf))
    }
}

#[derive(Debug, PartialEq)]
pub struct HeaderFirstByte(BitArr!(for 8, in Msb0, u8));

impl FromReadBytes for HeaderFirstByte {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
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

impl HeaderFirstByte {
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
