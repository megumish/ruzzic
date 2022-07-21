use std::io::Cursor;

use byteorder::{NetworkEndian, ReadBytesExt};
use ruzzic_common::read_bytes_to::{FromReadBytesWith, ReadBytesTo};

#[derive(Debug, PartialEq)]
pub struct Body {
    length: usize,
    value: StatusRequest,
}

#[derive(Debug, PartialEq)]
pub enum StatusRequest {
    Ocsp(OcspStatusRequest),
    Others(u8),
}

#[derive(Debug, PartialEq)]
pub struct OcspStatusRequest {
    responder_ids: Vec<ResponderId>,
    extensions: Vec<u8>,
    total_length: usize,
}

#[derive(Debug, PartialEq)]
pub struct ResponderId(Vec<u8>);

impl FromReadBytesWith<()> for Body {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let length = input.read_u16::<NetworkEndian>()? as usize;
        let mut input = {
            let mut buf = vec![0u8; length];
            input.read_exact(&mut buf)?;
            Cursor::new(buf)
        };
        let value = input.read_bytes_to()?;
        Ok(Body { length, value })
    }
}

impl FromReadBytesWith<()> for StatusRequest {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let status_type = input.read_u8()?;
        Ok(match status_type {
            1 => Self::Ocsp(input.read_bytes_to()?),
            x => Self::Others(x),
        })
    }
}

impl FromReadBytesWith<()> for OcspStatusRequest {
    fn from_read_bytes_with<R: std::io::Read>(
        input: &mut R,
        with: (),
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let (responder_ids, responder_ids_length) = {
            let length = input.read_u16::<NetworkEndian>()? as usize;
            let mut total_length = 0;
            let mut responder_ids = Vec::new();
            while total_length < length {
                let length = input.read_u16::<NetworkEndian>()? as usize;
                let mut responder_id = vec![0u8; length];
                input.read_exact(&mut responder_id);
                total_length += length;
                responder_ids.push(ResponderId(responder_id));
            }
            (responder_ids, total_length)
        };

        let (extensions, extensions_length) = {
            let length = input.read_u16::<NetworkEndian>()? as usize;
            let mut extensions = vec![0u8; length];
            input.read_exact(&mut extensions);
            (extensions, length)
        };

        Ok(Self {
            responder_ids,
            extensions,
            total_length: responder_ids_length + extensions_length,
        })
    }
}

impl Body {
    pub(crate) fn size_of(&self) -> usize {
        2 + self.length
    }
}
