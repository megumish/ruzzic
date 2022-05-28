use std::borrow::Cow;

use bitvec::prelude::*;
use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    connection::ConnectionID,
    read_bytes_to::{FromReadBytesWith, ReadBytesTo, ReadBytesToWith},
    FromReadBytes, Version,
};

use super::{packet_meta::PacketMeta, PacketNumber};

pub mod initial;
pub mod version_negotiation;

#[derive(Debug, PartialEq)]
pub struct LongHeaderMeta {
    first_byte: BitArr!(for 8, in Msb0, u8),
    version: Version,
}

// TODO: use ConnectionID struct instead of Vec
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionIDPair {
    pub destination_id: Vec<u8>,
    pub source_id: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub enum PacketType {
    Initial,
    ZeroRTT,
    Handshake,
    Retry,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Versions(Vec<Version>);

#[derive(Debug, Clone, PartialEq)]
pub enum LongHeader {
    VersionNegotiation(version_negotiation::Body),
    Initial(initial::Body),
}

impl LongHeader {
    pub(super) fn payload(&self) -> &[u8] {
        match self {
            LongHeader::VersionNegotiation(b) => b.payload(),
            LongHeader::Initial(b) => b.payload(),
        }
    }

    pub(super) fn destination_connection_id(&self) -> ConnectionID {
        match self {
            LongHeader::VersionNegotiation(b) => b.destination_connection_id(),
            LongHeader::Initial(b) => b.destination_connection_id(),
        }
    }

    pub(super) fn source_connection_id(&self) -> ConnectionID {
        match self {
            LongHeader::VersionNegotiation(b) => b.source_connection_id(),
            LongHeader::Initial(b) => b.source_connection_id(),
        }
    }

    pub(super) fn packet_number(&self) -> PacketNumber {
        match self {
            LongHeader::VersionNegotiation(b) => unreachable!(),
            LongHeader::Initial(b) => b.packet_number(),
        }
    }

    pub(super) fn raw_length(&self) -> usize {
        match self {
            LongHeader::VersionNegotiation(b) => b.raw_length(),
            LongHeader::Initial(b) => b.raw_length(),
        }
    }
}

impl FromReadBytesWith<&PacketMeta> for LongHeader {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
        meta: &PacketMeta,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        if meta.version() == Version(0) {
            return Ok(LongHeader::VersionNegotiation(input.read_bytes_to()?));
        }
        Ok(match meta.long_packet_type() {
            PacketType::Initial => LongHeader::Initial(input.read_bytes_to_with(meta)?),
            _ => unimplemented!(),
        })
    }
}

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

    pub fn raw_length(&self) -> usize {
        self.destination_id.len() + self.source_id.len() +
        // destination_id_length + source_id_length
        1 + 1
    }
}

impl FromReadBytesWith<()> for ConnectionIDPair {
    fn from_read_bytes_with<T: std::io::Read>(input: &mut T, _: ()) -> Result<Self, std::io::Error>
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
}

impl FromReadBytesWith<()> for Versions {
    fn from_read_bytes_with<T: std::io::Read>(input: &mut T, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Self::read_bytes(input))
    }
}

#[cfg(test)]
mod tests {
    use byteorder::ByteOrder;

    use super::*;
    use std::io::Cursor;

    #[test]
    fn connection_id_pairs() {
        let destination_id = [0x01];
        let destination_id_length = [destination_id.len() as u8];

        let source_id = [0x02, 0x11];
        let source_id_length = [source_id.len() as u8];

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
