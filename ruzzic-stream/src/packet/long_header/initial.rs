use std::io::{Cursor, Read};

use ruzzic_common::read_bytes_to::{FromReadBytesWith, ReadBytesTo};

use super::ConnectionIDPair;
use crate::{
    connection::{Connection, ConnectionID},
    endpoint_state::EndpointState,
    packet::{self, packet_meta::PacketMeta, PacketData, PacketNumber, PacketPayload},
    read_varint, Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub connection_id_pair: ConnectionIDPair,
    pub token: Token,
    pub packet_number: PacketNumber,
    pub packet_payload: PacketPayload,
}

impl FromReadBytesWith<&PacketMeta> for Body {
    fn from_read_bytes_with<R: Read>(
        input: &mut R,
        meta: &PacketMeta,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let connection_id_pair = input.read_bytes_to()?;
        let token = input.read_bytes_to()?;
        let remainder_length = read_varint(input).unwrap().to_u64();
        let mut remainder = vec![0u8; remainder_length as usize];
        input.read_exact(&mut remainder)?;
        let mut remainder_input = Cursor::new(remainder);
        let packet_number =
            PacketNumber::read_bytes_to(&mut remainder_input, meta.packet_number_length()).unwrap();
        let packet_payload = remainder_input.read_bytes_to().unwrap();
        Ok(Self {
            connection_id_pair,
            token,
            packet_number,
            packet_payload,
        })
    }
}

impl Body {
    pub(super) fn payload(&self) -> &[u8] {
        &self.packet_payload.0
    }

    pub(super) fn destination_connection_id(&self) -> ConnectionID {
        ConnectionID(self.connection_id_pair.destination_id.clone())
    }

    pub(super) fn source_connection_id(&self) -> ConnectionID {
        ConnectionID(self.connection_id_pair.source_id.clone())
    }

    pub(crate) fn token_raw_length(&self) -> usize {
        self.token.raw_length()
    }

    pub(super) fn packet_number(&self) -> PacketNumber {
        self.packet_number.clone()
    }

    pub(super) fn overwrite_packet_number(&mut self, packet_number: PacketNumber) {
        self.packet_number = packet_number;
    }

    pub(super) fn raw_length(&self, packet_number_length: usize) -> usize {
        let connection_id_pair_length = self.connection_id_pair.raw_length();
        let token_length = self.token.raw_length();
        let packet_data_length = PacketData {
            packet_number: &self.packet_number,
            packet_payload: &self.packet_payload,
        }
        .raw_length(packet_number_length);
        connection_id_pair_length + token_length + packet_data_length
    }

    pub(crate) fn update_payload(self, payload: PacketPayload) -> Self {
        Self {
            connection_id_pair: self.connection_id_pair,
            token: self.token,
            packet_number: self.packet_number,
            packet_payload: payload,
        }
    }

    pub(crate) fn new(connection: &Connection, endpoint_state: &EndpointState) -> Self {
        let connection_id_pair = ConnectionIDPair {
            destination_id: connection.destination_connection_id().to_vec(),
            source_id: connection.source_connection_id().to_vec(),
        };
        let token = connection.token().clone();
        let packet_number = endpoint_state.next_packet_number().clone();
        let packet_payload = connection.next_packet_payload();

        Self {
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
    use crate::{packet::packet_meta::FirstByte, u64_to_varint_exact_size, Version};
    use bitvec::prelude::*;
    use ruzzic_common::read_bytes_to::ReadBytesToWith;
    use std::io::Cursor;

    #[test]
    fn initial_packet() {
        let buf = {
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
        let mut input = Cursor::new(buf);

        let mut first_byte = bitarr![Msb0, u8; 0; 8];
        first_byte.store(0b11000000u8);
        let actual: Body = input
            .read_bytes_to_with(&PacketMeta {
                first_byte: FirstByte(first_byte),
                version: Version(1),
            })
            .unwrap();
        let expected = Body {
            connection_id_pair: ConnectionIDPair {
                destination_id: vec![0x01],
                source_id: vec![0x02, 0x11],
            },
            token: Token(vec![0x41]),
            packet_number: PacketNumber(0x01),
            packet_payload: PacketPayload(vec![0x00]),
        };
        assert_eq!(actual, expected);
    }
}
