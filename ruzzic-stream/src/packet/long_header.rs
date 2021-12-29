use std::{
    intrinsics::transmute,
    io::{Read, Seek},
};

use bitvec::prelude::*;
use byteorder::{BigEndian, ByteOrder, ReadBytesExt, WriteBytesExt};

pub struct LongHeaderMeta {
    first_byte: BitArr!(for 8, in Msb0, u8),
    version: u32,
}

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

impl<'a> LongHeaderMeta {
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

    pub fn read_bytes(buffer: &'a [u8]) -> Self {
        let mut first_byte = bitarr![Msb0, u8; 0; 8];
        first_byte.store(buffer[0]);
        let version: u32 =
            unsafe { transmute::<_, &u32>(&buffer[1..5] as *const [u8] as *const [u8; 4]) }.to_le();
        Self {
            first_byte,
            version,
        }
    }
}

const CONNECTION_ID_LENGTH: usize = 8;
impl ConnectionIDPair {
    pub fn read_bytes(buffer: &[u8]) -> Self {
        let destination_id_length = BigEndian::read_u64(&buffer[0..8]);
        let source_id_length_begin_offset = CONNECTION_ID_LENGTH + destination_id_length as usize;

        let source_id_length = BigEndian::read_u64(&buffer[0..8]);
        let next_content_begin_offset =
            source_id_length_begin_offset + CONNECTION_ID_LENGTH + source_id_length as usize;
        Self {
            destination_id: buffer[CONNECTION_ID_LENGTH..source_id_length_begin_offset].to_vec(),
            source_id: buffer
                [source_id_length_begin_offset + CONNECTION_ID_LENGTH..next_content_begin_offset]
                .to_vec(),
        }
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
        let input = [first_byte.load(), 0x00, 0x00, 0x00, 0x00];
        let long_header_meta = LongHeaderMeta::read_bytes(&input);
        assert_eq!(long_header_meta.header_form(), HeaderForm::Long);
        assert!(long_header_meta.is_valid());
        assert_eq!(long_header_meta.long_packet_type(), PacketType::Initial);
        assert_eq!(long_header_meta.version, 0x00000000);
    }

    #[test]
    fn connection_id_pairs() {
        let destination_id = [0x01];
        let mut destination_id_length = vec![];
        destination_id_length.write_u64::<BigEndian>(destination_id.len() as u64);

        let source_id = [0x02];
        let mut source_id_length = vec![];
        source_id_length.write_u64::<BigEndian>(source_id.len() as u64);

        let input = [
            &destination_id_length[..],
            &destination_id[..],
            &source_id_length[..],
            &source_id[..],
        ]
        .concat();

        let connection_id_pair = ConnectionIDPair::read_bytes(&input);
        assert_eq!(connection_id_pair.destination_id, &destination_id);
        assert_eq!(connection_id_pair.source_id, &source_id);
    }
}
