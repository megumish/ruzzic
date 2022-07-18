use std::io::Cursor;

use byteorder::{NetworkEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::{FromReadBytesWith, ReadBytesTo};

#[derive(Debug, PartialEq)]
pub struct Body {
    length: usize,
    value: HeartbeatMode,
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum HeartbeatMode {
    PeerAllowedToSend,
    PeerNotAllowedToSend,
    Others(u8),
}

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let length = input.read_u16::<NetworkEndian>()? as usize;
        let mut input = {
            let mut buf = vec![0u8; length as usize];
            input.read_exact(&mut buf)?;
            Cursor::new(buf)
        };
        let value = input.read_bytes_to()?;
        Ok(Self { length, value })
    }
}

impl FromReadBytesWith<()> for HeartbeatMode {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mode = input.read_u8()?;
        Ok(match mode {
            1 => HeartbeatMode::PeerAllowedToSend,
            2 => HeartbeatMode::PeerNotAllowedToSend,
            x => HeartbeatMode::Others(x),
        })
    }
}

impl Body {
    pub(crate) fn size_of(&self) -> usize {
        2 + self.length
    }
}
