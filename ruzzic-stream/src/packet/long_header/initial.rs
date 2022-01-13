use std::io::{Cursor, Read};

use super::{ConnectionIDPair, HeaderForm, LongHeaderMeta, Version, Versions};
use crate::{
    packet::{PacketNumber, PacketPayload},
    read_varint, ReadBytesTo, Token,
};

#[derive(Debug, PartialEq)]
pub struct InitialPacket {
    pub version: Version,
    pub connection_id_pair: ConnectionIDPair,
    pub token: Token,
    pub packet_number: PacketNumber,
    pub packet_payload: PacketPayload,
}

pub(super) trait HasPacketNumberLength {
    fn packet_number_length(&self) -> u16;
}

impl InitialPacket {
    // TODO: implement error handling
    pub fn read_bytes(buffer: &[u8]) -> Self {
        let mut cursor = Cursor::new(buffer);
        let meta: LongHeaderMeta = cursor.read_bytes_to().unwrap();
        let connection_id_pair = cursor.read_bytes_to().unwrap();
        let token = cursor.read_bytes_to().unwrap();
        let remainder_length = read_varint(&mut cursor).unwrap().to_u64();
        let mut remainder = vec![0u8; remainder_length as usize];
        cursor.read_exact(&mut remainder).unwrap();
        let mut remainder_cursor = Cursor::new(remainder);
        let packet_number =
            PacketNumber::read_bytes_to(&mut remainder_cursor, meta.packet_number_length())
                .unwrap();
        let packet_payload = remainder_cursor.read_bytes_to().unwrap();
        Self {
            version: meta.version,
            connection_id_pair,
            token,
            packet_number,
            packet_payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::u64_to_varint_exact_size;
    use bitvec::prelude::*;
    use byteorder::{BigEndian, ByteOrder};

    #[test]
    fn initial_packet() {
        let input = {
            let first_byte = bitarr![Msb0, u8;
            1, // Header Form
            1, // Fixed Bit]
            0, 0, // Packet Type (Initial)
            0, 0, // Reserved Bits
            0, 1 - 1, // Packet Number Length (define after)
            ];
            let version = [0x00, 0x00, 0x00, 0x00];

            let destination_id = [0x01];
            let destination_id_length = [destination_id.len() as u8];

            let source_id = [0x02, 0x11];
            let source_id_length = [source_id.len() as u8];

            let token = [0x41];
            let token_length = u64_to_varint_exact_size(token.len() as u64).to_bytes();

            let packet_number = [0x01];
            let packet_payload = [0x00];
            let remainder_length =
                u64_to_varint_exact_size(packet_number.len() as u64 + packet_payload.len() as u64)
                    .to_bytes();
            [
                &[first_byte.load()][..],
                &version[..],
                &destination_id_length[..],
                &destination_id[..],
                &source_id_length[..],
                &source_id[..],
                &token_length[..],
                &token[..],
                &remainder_length[..],
                &packet_number[..],
                &packet_payload[..],
            ]
            .concat()
        };

        let initial_packet = InitialPacket::read_bytes(&input);
        let expected = InitialPacket {
            version: Version(0x00),
            connection_id_pair: ConnectionIDPair {
                destination_id: vec![0x01],
                source_id: vec![0x02, 0x11],
            },
            token: Token(vec![0x41]),
            packet_number: PacketNumber(0x01),
            packet_payload: PacketPayload(vec![0x00]),
        };
        assert_eq!(initial_packet, expected);
    }

    #[test]
    fn neqo_client_initial_packet() {
        const NEQO_CLIENT_INITIAL_PACKET: &[u8] = &[
            192, 0, 0, 0, 1, 14, 235, 214, 70, 57, 212, 238, 63, 22, 27, 127, 99, 173, 162, 52, 0,
            0, 66, 5, 0, 6, 0, 66, 0, 1, 0, 1, 252, 3, 3, 171, 53, 10, 48, 142, 153, 180, 131, 197,
            209, 123, 107, 62, 157, 243, 176, 128, 238, 81, 111, 234, 177, 62, 111, 23, 132, 47,
            84, 60, 35, 153, 40, 0, 0, 6, 19, 1, 19, 3, 19, 2, 1, 0, 1, 205, 0, 23, 0, 0, 255, 1,
            0, 1, 0, 0, 10, 0, 20, 0, 18, 0, 29, 0, 23, 0, 24, 0, 25, 1, 0, 1, 1, 1, 2, 1, 3, 1, 4,
            0, 16, 0, 5, 0, 3, 2, 104, 51, 0, 5, 0, 5, 1, 0, 0, 0, 0, 0, 51, 0, 38, 0, 36, 0, 29,
            0, 32, 200, 183, 179, 86, 218, 88, 58, 207, 55, 163, 148, 23, 237, 195, 154, 106, 97,
            162, 213, 15, 169, 202, 41, 148, 161, 215, 37, 217, 215, 142, 165, 34, 0, 43, 0, 3, 2,
            3, 4, 0, 13, 0, 24, 0, 22, 4, 3, 5, 3, 6, 3, 2, 3, 8, 4, 8, 5, 8, 6, 4, 1, 5, 1, 6, 1,
            2, 1, 0, 45, 0, 2, 1, 1, 0, 28, 0, 2, 64, 1, 0, 57, 0, 67, 5, 4, 128, 16, 0, 0, 7, 4,
            128, 16, 0, 0, 9, 1, 16, 6, 4, 128, 16, 0, 0, 32, 1, 0, 15, 0, 14, 1, 8, 12, 0, 8, 1,
            16, 11, 1, 20, 1, 4, 128, 0, 117, 48, 192, 0, 0, 0, 255, 2, 222, 26, 2, 67, 232, 4, 8,
            255, 255, 255, 255, 255, 255, 255, 255, 106, 178, 0, 0, 21, 0, 246, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let initial_packet = InitialPacket::read_bytes(NEQO_CLIENT_INITIAL_PACKET);
        eprintln!("{:#x?}", initial_packet);
    }

    #[test]
    fn neqo_server_initial_packet() {
        const NEQO_SERVER_INITIAL_PACKET: &[u8] = &[
            192, 0, 0, 0, 1, 0, 10, 189, 96, 17, 245, 154, 105, 252, 108, 171, 44, 0, 64, 100, 0,
            2, 0, 0, 0, 0, 6, 0, 64, 90, 2, 0, 0, 86, 3, 3, 219, 98, 183, 101, 225, 209, 143, 84,
            159, 231, 81, 246, 36, 1, 52, 248, 222, 203, 11, 68, 30, 155, 62, 173, 174, 167, 185,
            90, 104, 45, 91, 10, 0, 19, 1, 0, 0, 46, 0, 51, 0, 36, 0, 29, 0, 32, 18, 139, 193, 217,
            226, 59, 133, 108, 95, 30, 210, 203, 91, 196, 57, 52, 155, 5, 36, 50, 96, 211, 110,
            174, 98, 245, 73, 178, 5, 87, 111, 106, 0, 43, 0, 2, 3, 4,
        ];
        let initial_packet = InitialPacket::read_bytes(NEQO_SERVER_INITIAL_PACKET);
        eprintln!("{:#x?}", initial_packet);
    }
}
