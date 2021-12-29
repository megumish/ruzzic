use super::{ConnectionIDPair, HeaderForm, LongHeaderMeta, Version, Versions};

#[derive(Debug, PartialEq)]
pub struct VersionNegotiationPacket {
    pub version: Version,
    pub connection_id_pair: ConnectionIDPair,
    pub supported_versions: Versions,
}

impl VersionNegotiationPacket {
    pub fn read_bytes(buffer: &[u8]) -> Self {
        let meta = LongHeaderMeta::read_bytes(&buffer[..LongHeaderMeta::SIZE]);
        let connection_id_pair = ConnectionIDPair::read_bytes(&buffer[LongHeaderMeta::SIZE..]);
        let supported_versions = Versions::read_bytes(
            &buffer[LongHeaderMeta::SIZE + connection_id_pair.real_length()..],
        );
        Self {
            version: meta.version,
            connection_id_pair,
            supported_versions,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let meta = LongHeaderMeta::new_for_version_negotiation(self.version);
        [
            &meta.to_bytes()[..],
            &self.connection_id_pair.to_bytes(),
            &self.supported_versions.to_bytes(),
        ]
        .concat()
    }

    pub fn new(
        version: Version,
        connection_id_pair: ConnectionIDPair,
        supported_versions: Versions,
    ) -> Self {
        Self {
            version,
            connection_id_pair,
            supported_versions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;
    use byteorder::{BigEndian, ByteOrder};

    #[test]
    fn version_negotiation_packet() {
        let input = {
            let first_byte = bitarr![Msb0, u8;
                1, // Header Form
                1, // Fixed Bit
                0, 0, // Packet Type
                0, 0, 0, 0 // Type-Specific Bits
            ];
            let version = [0x00, 0x00, 0x00, 0x00];

            let destination_id = [0x01];
            let mut destination_id_length = [0u8; 8];
            BigEndian::write_u64(&mut destination_id_length, destination_id.len() as u64);

            let source_id = [0x02, 0x11];
            let mut source_id_length = [0u8; 8];
            BigEndian::write_u64(&mut source_id_length, source_id.len() as u64);

            let versions = [0x01, 0x02]
                .iter()
                .map(|version| {
                    let mut buf = [0u8; 4];
                    BigEndian::write_u32(&mut buf, *version);
                    buf
                })
                .collect::<Vec<_>>()
                .concat();

            [
                &[first_byte.load()][..],
                &version[..],
                &destination_id_length[..],
                &destination_id[..],
                &source_id_length[..],
                &source_id[..],
                &versions[..],
            ]
            .concat()
        };

        let version_negotiation_packet = VersionNegotiationPacket::read_bytes(&input);
        let expected = VersionNegotiationPacket {
            version: Version(0x00),
            connection_id_pair: ConnectionIDPair {
                destination_id: vec![0x01],
                source_id: vec![0x02, 0x11],
            },
            supported_versions: Versions(vec![Version(0x01), Version(0x02)]),
        };
        assert_eq!(version_negotiation_packet.version, expected.version);
        assert_eq!(
            version_negotiation_packet.connection_id_pair,
            expected.connection_id_pair
        );
        assert_eq!(
            version_negotiation_packet.supported_versions,
            expected.supported_versions
        );
    }
}
