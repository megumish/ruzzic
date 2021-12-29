use super::{ConnectionIDPair, HeaderForm, LongHeaderMeta, Version, Versions};
use crate::{
    packet::{PacketNumber, PacketPayload},
    Token,
};

#[derive(Debug, PartialEq)]
pub struct InitialPacket {
    pub version: Version,
    pub connection_id_pair: ConnectionIDPair,
    pub token: Token,
    pub pakcet_number: PacketNumber,
    pub packet_payload: PacketPayload,
}

impl InitialPacket {
    pub fn read_bytes(buffer: &[u8]) -> Self {
        let meta = LongHeaderMeta::read_bytes(&buffer[..LongHeaderMeta::SIZE]);
        let connection_id_pair = ConnectionIDPair::read_bytes(&buffer[LongHeaderMeta::SIZE..]);
        let token =
            Token::read_bytes(&buffer[LongHeaderMeta::SIZE + connection_id_pair.real_length()..]);
    }
}
