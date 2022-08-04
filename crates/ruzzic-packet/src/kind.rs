use crate::packet::long_header::{initial::InitialPacket, LongHeaderPacket};

pub enum PacketKind<'a> {
    LongHeader(LongHeaderPacket<'a>),
    Initial(InitialPacket<'a>),
}
