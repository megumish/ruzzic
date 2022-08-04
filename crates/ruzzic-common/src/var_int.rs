pub struct VarInt(u64);

impl VarInt {
    #[cfg(feature = "std")]
    pub fn parse_and_get_raw_length<T>(buf: T, offset: usize) -> Result<(VarInt, usize), ParseError>
    where
        T: AsRef<[u8]>,
    {
        let buf = buf.as_ref();
        let top_byte = buf.get(offset).ok_or(ParseError::UnexpectedEnd(offset))?;
        let length: usize = 1 << (top_byte >> 6);
        let mut value = 0u64;
        for _ in 0..length {
            value =
                (value << 8) + *buf.get(offset).ok_or(ParseError::UnexpectedEnd(offset))? as u64;
        }
        Ok((VarInt(value), length))
    }

    pub fn u64(&self) -> u64 {
        if self.0 - (0b00 << 6) < (1 << 6) {
            self.0 - (0b00 << 6)
        } else if self.0 - (0b01 << 14) < (1 << 14) {
            self.0 - (0b01 << 14)
        } else if self.0 - (0b10 << 30) < (1 << 30) {
            self.0 - (0b10 << 30)
        } else if self.0 - (0b11 << 62) < (1 << 62) {
            self.0 - (0b11 << 62)
        } else {
            panic!("unsupported size");
        }
    }
}

#[cfg(feature = "std")]
pub enum ParseError {
    UnexpectedEnd(usize),
}
