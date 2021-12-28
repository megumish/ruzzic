use std::intrinsics::transmute;

use bitvec::prelude::*;

pub struct LongHeaderMeta {
    first_byte: BitArr!(for 8, in Msb0, u8),
    version: u32,
}

pub struct ConnectionIDPair<'a> {
    pub destination_id: &'a [u8],
    pub source_id: &'a [u8],
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
}
