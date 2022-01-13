use crate::{FromReadBytes, read_varint};

mod padding;
mod ping;

#[derive(Debug, PartialEq)]
enum Frame {
    Padding,
    Ping,
    Extension(u64),
}

impl FromReadBytes for Frame {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let frame_type = read_varint(input)?.to_u64();
        Ok(match frame_type {
            0x00 => Frame::Padding,
            0x01 => Frame::Ping,
            x => Frame::Extension(frame_type),
        })
    } 
}