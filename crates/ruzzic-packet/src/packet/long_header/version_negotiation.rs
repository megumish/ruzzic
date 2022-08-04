use ruzzic_common::QuicVersion;

use crate::packet::long_header::{self, LongHeaderPacket, LongHeaderPacketTransformError};

use self::supported_versions::{supported_versions, SupportedVersions};

mod supported_versions;

#[derive(Debug, PartialEq)]
pub struct VersionNegotiationPacket<'a> {
    first_byte: u8,
    destination_connection_id: &'a [u8],
    source_connection_id: &'a [u8],
    supported_versions: Vec<QuicVersion>,
}

impl<'a> TryFrom<&'a LongHeaderPacket<'a>> for VersionNegotiationPacket<'a> {
    type Error = LongHeaderPacketTransformError;

    fn try_from(packet: &'a LongHeaderPacket<'a>) -> Result<Self, Self::Error> {
        if is_version_negotiation(packet) {
            let type_specific_payload = packet.type_specific_payload();
            let position = 0;

            let SupportedVersions {
                value: supported_versions,
                position_after: _,
            } = supported_versions(type_specific_payload, position)?;

            Ok(Self {
                first_byte: packet.first_byte(),
                destination_connection_id: packet.destination_connection_id(),
                source_connection_id: packet.source_connection_id(),
                supported_versions,
            })
        } else {
            Err(Self::Error::NotThisKind(
                long_header::KindOfPacket::VersionNegotiation,
            ))
        }
    }
}

pub fn is_version_negotiation(long_header_packet: &LongHeaderPacket) -> bool {
    long_header_packet.version() == QuicVersion::VersionNegotiation
}

#[cfg(test)]
mod tests {
    use ruzzic_common::QuicVersion;

    use crate::packet::long_header::{
        KindOfPacket, LongHeaderPacket, LongHeaderPacketTransformError,
    };

    use super::VersionNegotiationPacket;

    #[test]
    fn simple() -> Result<(), LongHeaderPacketTransformError> {
        let input: &LongHeaderPacket = &LongHeaderPacket::new(
            0b1100_0000,
            QuicVersion::VersionNegotiation,
            &[],
            &[],
            &[
                0, 0, 0, 1, // Version RFC9000
                0, 0, 0, 2, // Version 0x00000002
            ],
        );

        let initial_packet: VersionNegotiationPacket = input.try_into()?;
        assert_eq!(
            initial_packet,
            VersionNegotiationPacket {
                first_byte: 0b1100_0000,
                destination_connection_id: &[],
                source_connection_id: &[],
                supported_versions: vec![QuicVersion::Rfc9000, QuicVersion::Others(2)]
            }
        );
        Ok(())
    }

    #[test]
    fn not_version_negotiation() {
        let input: &LongHeaderPacket =
            &LongHeaderPacket::new(0b1101_0000, QuicVersion::Rfc9000, &[], &[], &[0, 2, 0, 0]);

        let result: Result<VersionNegotiationPacket, _> = input.try_into();
        assert_eq!(
            result,
            Err(LongHeaderPacketTransformError::NotThisKind(
                KindOfPacket::VersionNegotiation
            ))
        );
    }
}
