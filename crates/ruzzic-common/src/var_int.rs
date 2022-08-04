use crate::next_bytes::NextBytes;

pub struct VarInt(u64);

impl VarInt {
    pub fn parse<T, E>(
        buf: T,
        offset: &mut usize,
        unexpected_end_error: &impl Fn(usize) -> E,
    ) -> Result<VarInt, E>
    where
        T: AsRef<[u8]>,
    {
        let buf = buf.as_ref();
        let mut temp_offset = offset.clone();
        let top_byte = buf.next_byte(&mut 0, unexpected_end_error)?;
        let length = 1 << (top_byte >> 6);
        let mut value = 0u64;
        for mut i in 0..length {
            value = (value << 8) + buf.next_byte(&mut i, unexpected_end_error)? as u64;
        }
        Ok(VarInt(value))
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
