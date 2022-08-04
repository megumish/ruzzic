use ruzzic_common::{next_bytes::NextBytes, var_int::VarInt, QuicVersion};

use crate::long_header::{LongHeaderPacket, LongHeaderPacketTransformError};

pub struct InitialPacket<'a> {
    first_byte: u8,
    version: QuicVersion,
    destination_connection_id: &'a [u8],
    source_connection_id: &'a [u8],
    token: &'a [u8],
    packet_number: u32,
    payload: &'a [u8],
}

impl<'a> TryFrom<&'a LongHeaderPacket<'a>> for InitialPacket<'a> {
    type Error = LongHeaderPacketTransformError<Self>;

    fn try_from(packet: &'a LongHeaderPacket<'a>) -> Result<Self, Self::Error> {
        if is_initial(packet) {
            let type_specific_payload = packet.type_specific_payload();
            let position = &mut 0;

            let token_length =
                VarInt::parse(type_specific_payload, position, &Self::Error::UnexpectedEnd)?.u64()
                    as usize;

            // TODO: I really want to use next_bytes,
            // but the lifetime definition does not work, so I am writing inline.
            let token = type_specific_payload
                .get(*position..*position + token_length)
                .ok_or(Self::Error::UnexpectedEnd(*position))?;
            *position += token_length;

            let remain_length =
                VarInt::parse(type_specific_payload, position, &Self::Error::UnexpectedEnd)?.u64()
                    as usize;

            let packet_number_length = packet.packet_number_length();
            let packet_number = {
                let buf = type_specific_payload.next_bytes(
                    packet_number_length as usize,
                    position,
                    &Self::Error::UnexpectedEnd,
                )?;
                let mut packet_number = 0u32;
                for b in buf {
                    packet_number = (packet_number << 8) + *b as u32;
                }
                packet_number
            };

            let payload_length = remain_length - packet_number_length;
            // TODO: I really want to use next_bytes,
            // but the lifetime definition does not work, so I am writing inline.
            let payload = type_specific_payload
                .get(*position..*position + payload_length)
                .ok_or(Self::Error::UnexpectedEnd(*position))?;

            Ok(Self {
                first_byte: packet.first_byte(),
                version: packet.version(),
                destination_connection_id: packet.destination_connection_id(),
                source_connection_id: packet.source_connection_id(),
                token,
                packet_number,
                payload,
            })
        } else {
            Err(Self::Error::NotThisKind)
        }
    }
}

pub fn is_initial(long_header_packet: &LongHeaderPacket) -> bool {
    (long_header_packet.first_byte() & 0b0001_0000) != 0
}
