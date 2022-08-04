use crate::{initial::InitialPacket, long_header::LongHeaderPacket, short_header::OneRttPacket};

pub enum PacketKind<'a> {
    LongHeader(LongHeaderPacket<'a>),
    Initial(InitialPacket<'a>),

    OneRtt(OneRttPacket<'a>),
}
