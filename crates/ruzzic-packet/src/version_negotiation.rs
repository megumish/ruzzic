use ruzzic_common::{next_bytes::NextBytes, QuicVersion};

use crate::long_header::{LongHeaderPacket, LongHeaderPacketTransformError};

pub struct VersionNegotiationPacket<'a> {
    first_byte: u8,
    destination_connection_id: &'a [u8],
    source_connection_id: &'a [u8],
    supported_versions: Vec<QuicVersion>,
}

impl<'a> TryFrom<&'a LongHeaderPacket<'a>> for VersionNegotiationPacket<'a> {
    type Error = LongHeaderPacketTransformError<Self>;

    fn try_from(packet: &'a LongHeaderPacket<'a>) -> Result<Self, Self::Error> {
        if is_version_negotiation(packet) {
            let type_specific_payload = packet.type_specific_payload();
            let position = &mut 0;

            let supported_versions_length = type_specific_payload
                .len()
                .checked_div_euclid(32)
                .ok_or(Self::Error::UnexpectedEnd(*position))?;
            let mut supported_versions = Vec::new();
            for _ in 0..supported_versions_length {
                let buf =
                    type_specific_payload.next_bytes(4, position, &Self::Error::UnexpectedEnd)?;
                let mut version = 0u32;
                for b in buf {
                    version = (version << 8) + *b as u32;
                }
                supported_versions.push(version.into());
            }
            Ok(Self {
                first_byte: packet.first_byte(),
                destination_connection_id: packet.destination_connection_id(),
                source_connection_id: packet.source_connection_id(),
                supported_versions,
            })
        } else {
            Err(Self::Error::NotThisKind)
        }
    }
}

pub fn is_version_negotiation(long_header_packet: &LongHeaderPacket) -> bool {
    long_header_packet.version() == QuicVersion::VersionNegotiation
}
