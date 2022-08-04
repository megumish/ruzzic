use ruzzic_common::QuicVersion;

use crate::packet::long_header::{self, LongHeaderPacket, LongHeaderPacketTransformError};

mod packet_number;
mod payload;
mod remain_length;
mod token;

#[derive(Debug, PartialEq)]
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
    type Error = LongHeaderPacketTransformError;

    fn try_from(packet: &'a LongHeaderPacket<'a>) -> Result<Self, Self::Error> {
        if is_initial(packet) {
            let type_specific_payload = packet.type_specific_payload();
            let position = 0;

            let token::Token {
                value: token,
                position_after: position,
            } = token::token(type_specific_payload, position)?;

            let remain_length::RemainLength {
                value: remain_length,
                position_after: position,
            } = remain_length::remain_length(type_specific_payload, position)?;

            let packet_number_length = packet.packet_number_length();
            let packet_number::PacketNumber {
                value: packet_number,
                position_after: position,
            } = packet_number::packet_number(
                type_specific_payload,
                position,
                packet_number_length,
            )?;

            let payload::Payload {
                value: payload,
                position_after: _,
            } = payload::payload(
                type_specific_payload,
                position,
                remain_length - packet_number_length,
            )?;

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
            Err(Self::Error::NotThisKind(long_header::KindOfPacket::Initial))
        }
    }
}

pub fn is_initial(long_header_packet: &LongHeaderPacket) -> bool {
    ((long_header_packet.first_byte() & 0b0011_0000) >> 4) == 0
}

#[cfg(test)]
mod tests {
    use ruzzic_common::QuicVersion;

    use crate::packet::long_header::{
        KindOfPacket, LongHeaderPacket, LongHeaderPacketTransformError,
    };

    use super::InitialPacket;

    #[test]
    fn simple() -> Result<(), LongHeaderPacketTransformError> {
        let input: &LongHeaderPacket =
            &LongHeaderPacket::new(0b1100_0000, QuicVersion::Rfc9000, &[], &[], &[0, 2, 0, 0]);

        let initial_packet: InitialPacket = input.try_into()?;
        assert_eq!(
            initial_packet,
            InitialPacket {
                first_byte: 0b1100_0000,
                version: QuicVersion::Rfc9000,
                destination_connection_id: &[],
                source_connection_id: &[],
                token: &[],
                packet_number: 0,
                payload: &[0],
            }
        );
        Ok(())
    }

    #[test]
    fn not_initial() {
        let input: &LongHeaderPacket =
            &LongHeaderPacket::new(0b1101_0000, QuicVersion::Rfc9000, &[], &[], &[0, 2, 0, 0]);

        let result: Result<InitialPacket, _> = input.try_into();
        assert_eq!(
            result,
            Err(LongHeaderPacketTransformError::NotThisKind(
                KindOfPacket::Initial
            ))
        );
    }
}
