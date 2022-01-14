use bitvec::{prelude::*, slice::BitSlice};

use crate::{
    read_varint,
    stream::{StreamData, StreamID},
    VarInt,
};

#[derive(Debug, PartialEq)]
pub struct Body {
    stream_id: StreamID,
    offset: Option<VarInt>,
    data: StreamData,
    is_fin: bool,
}

impl Body {
    // TODO: error handling when access flags
    pub fn read_bytes_to<T: std::io::Read>(
        input: &mut T,
        flags: &BitSlice<Msb0, u8>,
    ) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let stream_id = read_varint(input)?;
        let offset = if flags[0] {
            Some(read_varint(input)?)
        } else {
            None
        };
        let mut buf = Vec::new();
        if flags[1] {
            let length = read_varint(input)?;
            buf.resize(length.to_u64() as usize, 0);
            input.read_exact(&mut buf)?;
        } else {
            let _ = input.read_to_end(&mut buf)?;
        };
        let is_fin = flags[2];
        Ok(Self {
            stream_id: StreamID(stream_id.to_u64()),
            offset,
            data: StreamData(buf),
            is_fin,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use bitvec::prelude::*;

    use super::Body;
    use crate::{
        stream::{StreamData, StreamID},
        VarInt,
    };

    #[test]
    fn stream_with_all() {
        let flags = bitarr![Msb0, u8; 1, 1, 1];
        let buf = [
            0, // StreamID
            0, // Offset
            1, // Data Length
            0, // Data
        ];
        let mut input = Cursor::new(buf);
        let actual: Body = Body::read_bytes_to(&mut input, &flags[..]).unwrap();
        let expected = Body {
            stream_id: StreamID(0),
            offset: Some(VarInt(0)),
            is_fin: true,
            data: StreamData(vec![0]),
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn stream_without_offset() {
        let mut flags = bitarr![Msb0, u8; 1];
        flags.store_le(0b00001011u8);
        let buf = [
            0, // StreamID
            1, // Data Length
            0, // Data
        ];
        let mut input = Cursor::new(buf);
        let actual: Body = Body::read_bytes_to(&mut input, &flags[5..]).unwrap();
        let expected = Body {
            stream_id: StreamID(0),
            offset: None,
            is_fin: true,
            data: StreamData(vec![0]),
        };
        assert_eq!(actual, expected);
    }
}
