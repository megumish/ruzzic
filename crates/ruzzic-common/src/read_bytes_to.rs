#![cfg(feature = "std")]
use std::io::{Error, Read};

pub trait ReadBytesToWith<With>: Read {
    fn read_bytes_to_with<T>(&mut self, with: With) -> Result<T, Error>
    where
        Self: Sized,
        T: FromReadBytesWith<With>,
    {
        T::from_read_bytes_with(self, with)
    }
}

impl<R, T> ReadBytesToWith<T> for R where R: Read {}

pub trait FromReadBytesWith<With> {
    fn from_read_bytes_with<R: Read>(input: &mut R, with: With) -> Result<Self, Error>
    where
        Self: Sized;
}

pub trait ReadBytesTo: ReadBytesToWith<()> {
    fn read_bytes_to<T>(&mut self) -> Result<T, Error>
    where
        Self: Sized,
        T: FromReadBytes,
    {
        self.read_bytes_to_with(())
    }
}

impl<R> ReadBytesTo for R where R: ReadBytesToWith<()> {}

pub trait FromReadBytes: FromReadBytesWith<()> {
    fn from_read_bytes<R: Read>(input: &mut R) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Self::from_read_bytes_with(input, ())
    }
}

impl<R> FromReadBytes for R where R: FromReadBytesWith<()> {}
