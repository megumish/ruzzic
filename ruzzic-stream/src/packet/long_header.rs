use bitvec::prelude::*;
use byteorder::{BigEndian, ByteOrder, ReadBytesExt, WriteBytesExt};
use std::{io::Cursor, mem::transmute};

use crate::{FromReadBytes, Version};

use super::HeaderFirstByte;

pub mod initial;
pub mod version_negotiation;

#[derive(Debug, PartialEq)]
pub struct LongHeaderMeta {
    first_byte: BitArr!(for 8, in Msb0, u8),
    version: Version,
}

// TODO: use ConnectionID struct instead of Vec
#[derive(Debug, PartialEq)]
pub struct ConnectionIDPair {
    pub destination_id: Vec<u8>,
    pub source_id: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum HeaderForm {
    Short,
    Long,
}

#[derive(Debug, PartialEq)]
pub enum PacketType {
    Initial,
    ZeroRTT,
    Handshake,
    Retry,
}

#[derive(Debug, PartialEq)]
pub struct Versions(Vec<Version>);

#[derive(Debug, PartialEq)]
pub struct LongHeader {
    first_byte: HeaderFirstByte,
    version: Version,
}

impl<'a> LongHeaderMeta {
    const SIZE: usize = 1 + 4;

    pub(self) fn new_for_version_negotiation(version: Version) -> Self {
        let first_byte = bitarr![Msb0, u8;
            1, // Header Form
            1, // Fixed Bit (but not used)
            0, 0, // Packet Type (but not used)
            0, 0, 0, 0 // Type-Specific Bits (but not used)
        ];
        Self {
            first_byte,
            version: version,
        }
    }

    pub fn header_form(&self) -> HeaderForm {
        match self.first_byte[0] {
            true => HeaderForm::Long,
            false => HeaderForm::Short,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.first_byte[1]
    }

    pub fn long_packet_type(&self) -> PacketType {
        match self.first_byte[2..4].load::<u8>() {
            0 => PacketType::Initial,
            1 => PacketType::ZeroRTT,
            2 => PacketType::Handshake,
            3 => PacketType::Retry,
            _ => unreachable!("this must be 2bit value"),
        }
    }

    pub fn read_fixed_bytes(buffer: &'a [u8]) -> Self {
        let mut first_byte = bitarr![Msb0, u8; 0; 8];
        first_byte.store(buffer[0]);
        let version: Version = Version(BigEndian::read_u32(&buffer[1..5]));
        Self {
            first_byte,
            version,
        }
    }

    pub(self) fn to_bytes(&self) -> [u8; 5] {
        [&[self.first_byte.load()], &self.version.to_bytes()[..]]
            .concat()
            .try_into()
            .unwrap()
    }
}

impl initial::HasPacketNumberLength for LongHeaderMeta {
    fn packet_number_length(&self) -> u16 {
        self.first_byte[6..8].load_be::<u16>() + 1
    }
}

impl FromReadBytes for LongHeaderMeta {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut buf = [0u8; 5];
        input.read_exact(&mut buf)?;
        Ok(Self::read_fixed_bytes(&buf))
    }
}

const CONNECTION_ID_LENGTH: usize = 8;
impl ConnectionIDPair {
    // TODO: do error handling
    pub fn read_bytes(input: &mut impl std::io::Read) -> Self {
        let destination_id_length = input.read_u8().unwrap();
        let mut destination_id = vec![0u8; destination_id_length as usize];
        input.read_exact(&mut destination_id).unwrap();

        let source_id_length = input.read_u8().unwrap();
        let mut source_id = vec![0u8; source_id_length as usize];
        input.read_exact(&mut source_id).unwrap();

        Self {
            destination_id,
            source_id,
        }
    }

    pub(self) fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.write_u64::<BigEndian>(self.destination_id.len() as u64)
            .unwrap();
        buf.extend_from_slice(&self.destination_id);
        buf.write_u64::<BigEndian>(self.source_id.len() as u64)
            .unwrap();
        buf.extend_from_slice(&self.source_id);
        buf
    }

    pub fn real_length(&self) -> usize {
        CONNECTION_ID_LENGTH * 2 + self.destination_id.len() + self.source_id.len()
    }
}

impl FromReadBytes for ConnectionIDPair {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Self::read_bytes(input))
    }
}

impl Versions {
    pub fn read_bytes(input: &mut impl std::io::Read) -> Self {
        let mut versions = Vec::new();
        while let Ok(raw_version) = input.read_u32::<BigEndian>() {
            versions.push(Version(raw_version))
        }

        Self(versions)
    }

    pub(self) fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        for version in &self.0 {
            buf.write_u32::<BigEndian>(version.0).unwrap();
        }
        buf
    }
}

impl FromReadBytes for Versions {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Self::read_bytes(input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_header_meta() {
        let first_byte = bitarr![Msb0, u8;
            1, // Header Form
            1, // Fixed Bit
            0, 0, // Packet Type
            0, 0, 0, 0 // Type-Specific Bits
        ];
        let input = [first_byte.load(), 0x00, 0x00, 0x00, 0x01];
        let long_header_meta = LongHeaderMeta::read_fixed_bytes(&input);
        assert_eq!(long_header_meta.header_form(), HeaderForm::Long);
        assert!(long_header_meta.is_valid());
        assert_eq!(long_header_meta.long_packet_type(), PacketType::Initial);
        assert_eq!(long_header_meta.version, Version(0x00000001));
    }

    #[test]
    fn connection_id_pairs() {
        let destination_id = [0x01];
        let mut destination_id_length = [destination_id.len() as u8];

        let source_id = [0x02, 0x11];
        let mut source_id_length = [source_id.len() as u8];

        let mut input = Cursor::new(
            [
                &destination_id_length[..],
                &destination_id[..],
                &source_id_length[..],
                &source_id[..],
            ]
            .concat(),
        );

        let connection_id_pair = ConnectionIDPair::read_bytes(&mut input);
        assert_eq!(connection_id_pair.destination_id, &destination_id);
        assert_eq!(connection_id_pair.source_id, &source_id);
    }

    #[test]
    fn versions() {
        let mut input = Cursor::new(
            [0x01, 0x02]
                .iter()
                .map(|version| {
                    let mut buf = [0u8; 4];
                    BigEndian::write_u32(&mut buf, *version);
                    buf
                })
                .collect::<Vec<_>>()
                .concat(),
        );

        let versions = Versions::read_bytes(&mut input);
        assert_eq!(versions, Versions(vec![Version(0x01), Version(0x02)]));
    }
}
