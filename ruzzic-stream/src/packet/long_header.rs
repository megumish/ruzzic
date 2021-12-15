use bitvec::prelude::*;

pub struct LongHeader {
    first_byte: BitArr!(for 8, in Msb0, u8),
}

pub enum HeaderForm {
    Short,
    Long,
}

pub enum PacketType {
    Initial,
    ZeroRTT,
    Handshake,
    Retry,
}

impl LongHeader {
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
}
