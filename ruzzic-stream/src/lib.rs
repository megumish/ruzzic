#[macro_use]
extern crate futures;

mod packet;

#[cfg(test)]
mod tests;

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
