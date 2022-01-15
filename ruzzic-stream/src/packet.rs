use byteorder::{BigEndian, ByteOrder};

use crate::{
    read_bytes_to::{FromReadBytesWith, ReadBytesTo, ReadBytesToWith},
    FromReadBytes,
};

use self::{long_header::LongHeader, packet_meta::PacketMeta};

mod long_header;
pub mod packet_meta;

#[derive(Debug, PartialEq)]
pub struct Packet {
    meta: PacketMeta,
    body: PacketBody,
}

impl FromReadBytesWith<()> for Packet {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let meta = input.read_bytes_to()?;
        let body = input.read_bytes_to_with(&meta)?;
        Ok(Self { meta, body })
    }
}

#[derive(Debug, PartialEq)]
pub enum PacketBody {
    Long(LongHeader),
}

#[derive(Debug, PartialEq)]
pub enum PacketBodyType {
    Long,
    Short,
}

impl FromReadBytesWith<&PacketMeta> for PacketBody {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
        meta: &PacketMeta,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        match meta.get_type() {
            PacketBodyType::Long => Ok(PacketBody::Long(input.read_bytes_to_with(meta)?)),
            PacketBodyType::Short => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PacketType {
    Initial,
    ZeroRTT,
    Handshake,
    Retry,
}

#[derive(Debug, PartialEq)]
pub struct PacketNumber(pub(crate) u32);

impl PacketNumber {
    // TODO: check integer casting
    pub fn read_bytes_to(
        input: &mut impl std::io::Read,
        length: u16,
    ) -> Result<Self, std::io::Error> {
        let mut buf = vec![0u8; length as usize];
        input.read_exact(&mut buf)?;
        Ok(PacketNumber(
            BigEndian::read_uint(&buf, length as usize) as u32
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct PacketPayload(Vec<u8>);

impl FromReadBytes for PacketPayload {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut buf = Vec::new();
        input.read_to_end(&mut buf)?;
        Ok(PacketPayload(buf))
    }
}

#[cfg(test)]
mod tests {
    use crate::read_bytes_to::ReadBytesTo;

    use super::*;

    use std::io::Cursor;

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

        let mut input = Cursor::new(NEQO_CLIENT_INITIAL_PACKET);
        let initial_packet: Packet = input.read_bytes_to().unwrap();
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

        let mut input = Cursor::new(NEQO_SERVER_INITIAL_PACKET);
        let initial_packet: Packet = input.read_bytes_to().unwrap();
        eprintln!("{:#x?}", initial_packet);
    }
}
