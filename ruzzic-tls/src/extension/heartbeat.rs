use byteorder::ReadBytesExt;
use ruzzic_common::read_bytes_to::{FromReadBytesWith, ReadBytesTo};

#[derive(Debug)]
pub struct Body {
    mode: HeartbeatMode,
}

#[derive(Debug)]
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
        Ok(Body {
            mode: input.read_bytes_to()?,
        })
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
        1
    }
}
