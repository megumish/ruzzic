use byteorder::{BigEndian, ByteOrder};

use crate::{
    read_bytes_to::{FromReadBytesWith, ReadBytesTo, ReadBytesToWith},
    size_of_varint, FromReadBytes,
};

use self::{long_header::LongHeader, packet_meta::PacketMeta};

mod long_header;
pub mod packet_meta;

#[derive(Debug, PartialEq)]
pub struct Packet {
    meta: PacketMeta,
    body: PacketBody,
}

impl FromReadBytesWith<()> for Packet {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let meta = input.read_bytes_to()?;
        let body = input.read_bytes_to_with(&meta)?;
        Ok(Self { meta, body })
    }
}

impl Packet {
    pub fn payload(&self) -> &[u8] {
        self.body.payload()
    }

    pub fn raw_length(&self) -> usize {
        self.meta.raw_length() + self.body.raw_length()
    }
}

#[derive(Debug, PartialEq)]
pub enum PacketBody {
    Long(LongHeader),
}

impl PacketBody {
    fn payload(&self) -> &[u8] {
        match self {
            PacketBody::Long(b) => b.payload(),
            _ => unimplemented!(),
        }
    }

    fn raw_length(&self) -> usize {
        match self {
            PacketBody::Long(lh) => lh.raw_length(),
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PacketBodyType {
    Long,
    Short,
}

impl FromReadBytesWith<&PacketMeta> for PacketBody {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
        meta: &PacketMeta,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        match meta.get_type() {
            PacketBodyType::Long => Ok(PacketBody::Long(input.read_bytes_to_with(meta)?)),
            _ => unimplemented!(),
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
        length: u8,
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

struct PacketData<'a> {
    pub packet_number: &'a PacketNumber,
    pub packet_payload: &'a PacketPayload,
}

impl<'a> PacketData<'a> {
    fn raw_length(&self) -> usize {
        let Self {
            packet_number,
            packet_payload,
        } = self;
        let packet_data_length = packet_payload.0.len()
            + if packet_number.0 <= 0xff {
                1
            } else if packet_number.0 <= 0xffff {
                2
            } else {
                4
            };
        size_of_varint(packet_data_length as u64) + packet_data_length
    }
}

#[cfg(test)]
mod neqo_tests;
