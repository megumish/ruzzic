use bitvec::macros::internal::funty::IsInteger;
use byteorder::{BigEndian, ByteOrder, NativeEndian, NetworkEndian, ReadBytesExt, WriteBytesExt};
use derive_more::{From, Into};
use std::{io::Cursor, mem::size_of, slice::from_raw_parts};

use self::read_bytes_to::FromReadBytesWith;

mod connection;
mod frame;
pub mod packet;
mod read_bytes_to;
mod stream;

// https://www.rfc-editor.org/rfc/rfc9000.html#name-variable-length-integer-enc
#[derive(Debug, Into, From, PartialEq)]
struct VarInt(u64);

fn read_varint(input: &mut impl std::io::Read) -> Result<VarInt, std::io::Error> {
    let top_byte = input.read_u8()?;
    let length = 1 << (top_byte >> 6);

    let mut buf = vec![0; length - 1];
    input.read_exact(&mut buf)?;
    let mut cursor = Cursor::new([&[top_byte], &buf[..]].concat());
    Ok(match length {
        1 => (top_byte as u64).into(),
        2 => (cursor.read_u16::<BigEndian>()? as u64).into(),
        4 => (cursor.read_u32::<BigEndian>()? as u64).into(),
        8 => (cursor.read_u64::<BigEndian>()? as u64).into(),
        _ => unreachable!("unexpected length"),
    })
}

// TODO: support error handling
#[allow(dead_code)]
fn to_varint<T>(i: &T) -> VarInt {
    let i_slice = unsafe { from_raw_parts(i as *const _ as *const _, size_of::<T>()) };
    let (msb, i) = match size_of::<T>() {
        1 => (0b00, i_slice[0] as u64),
        2 => (0b01, (NativeEndian::read_u16(i_slice)) as u64),
        4 => (0b10, (NativeEndian::read_u32(i_slice)) as u64),
        8 => (0b11, (NativeEndian::read_u64(i_slice))),
        _ => panic!("unsupported size"),
    };
    (msb << (size_of::<T>() * 8 - 2))
        .checked_add(i)
        .unwrap()
        .into()
}

// TODO: implement error handling
#[allow(dead_code)]
fn u64_to_varint_exact_size(i: u64) -> VarInt {
    if i < (1 << 6) {
        to_varint(&(i as u8))
    } else if i < (1 << 14) {
        to_varint(&(i as u16))
    } else if i < (1 << 30) {
        to_varint(&(i as u32))
    } else if i < (1 << 62) {
        to_varint(&(i as u64))
    } else {
        panic!("unsupported size");
    }
}

impl VarInt {
    #[allow(dead_code)]
    fn byte_size(&self) -> usize {
        if self.0 - (0b00 << 6) < (1 << 6) {
            1
        } else if self.0 - (0b01 << 14) < (1 << 14) {
            2
        } else if self.0 - (0b10 << 30) < (1 << 30) {
            4
        } else if self.0 - (0b11 << 62) < (1 << 62) {
            8
        } else {
            panic!("unsupported size");
        }
    }

    fn to_u64(&self) -> u64 {
        if self.0 - (0b00 << 6) < (1 << 6) {
            self.0 - (0b00 << 6)
        } else if self.0 - (0b01 << 14) < (1 << 14) {
            self.0 - (0b01 << 14)
        } else if self.0 - (0b10 << 30) < (1 << 30) {
            self.0 - (0b10 << 32)
        } else if self.0 - (0b11 << 62) < (1 << 62) {
            self.0 - (0b11 << 62)
        } else {
            panic!("unsupported size");
        }
    }

    #[allow(dead_code)]
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![0; self.byte_size()];
        BigEndian::write_uint(&mut buf, self.0, self.byte_size());
        buf
    }

    // TODO: support error handling
    #[allow(dead_code)]
    fn write_varint(&self, output: &mut impl std::io::Write) -> Result<(), std::io::Error> {
        output.write_uint::<BigEndian>(self.0, self.byte_size())?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Token(Vec<u8>);

impl Token {
    // TODO: support error handling
    pub fn read_bytes(input: &mut impl std::io::Read) -> Self {
        let length = read_varint(input).unwrap();
        let mut buf = vec![0; length.to_u64() as usize];
        input.read_exact(&mut buf).unwrap();
        Token(buf)
    }
}

impl FromReadBytes for Token {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Self::read_bytes(input))
    }
}

trait ReadBytesTo: std::io::Read {
    fn read_bytes_to<T>(&mut self) -> Result<T, std::io::Error>
    where
        Self: Sized,
        T: FromReadBytes,
    {
        T::from_read_bytes(self)
    }
}

impl<R> ReadBytesTo for R where R: std::io::Read {}

trait FromReadBytes {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq)]
pub struct ApplicationProtocolErrorCode(pub(crate) u64);

#[repr(transparent)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Version(u32);

impl FromReadBytesWith<()> for Version {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Self(input.read_u32::<NetworkEndian>()?))
    }
}

impl Version {
    #[allow(dead_code)]
    pub(self) fn to_bytes(&self) -> [u8; 4] {
        let mut buf = [0u8; 4];
        BigEndian::write_u32(&mut buf, self.0);
        buf
    }
}
