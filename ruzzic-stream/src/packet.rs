use byteorder::{BigEndian, ByteOrder};

use crate::{
    read_bytes_to::{FromReadBytesWith, ReadBytesToWith},
    FromReadBytes,
};

use self::{long_header::LongHeader, packet_meta::PacketMeta};

mod long_header;
mod packet_meta;

#[derive(Debug, PartialEq)]
pub struct Packet {
    meta: PacketMeta,
    body: PacketBody,
}

#[derive(Debug, PartialEq)]
pub enum PacketBody {
    Long(LongHeader),
}

#[derive(Debug, PartialEq)]
pub enum PacketBodyType {
    Long,
}

impl FromReadBytesWith<PacketMeta> for PacketBody {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
        meta: PacketMeta,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        match meta.get_type() {
            PacketBodyType::Long => Ok(PacketBody::Long(input.read_bytes_to_with(meta)?)),
        }
    }
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
