use std::slice::SliceIndex;

pub trait NextBytes<'a> {
    fn next_byte<E>(
        &self,
        offset: &mut usize,
        unexpected_end_error: impl Fn(usize) -> E,
    ) -> Result<u8, E>;

    fn next_bytes<E>(
        &self,
        length: usize,
        offset: &mut usize,
        unexpected_end_error: impl Fn(usize) -> E,
    ) -> Result<&[u8], E>;
}

impl<'a> NextBytes<'a> for &'a [u8] {
    fn next_byte<E>(
        &self,
        offset: &mut usize,
        unexpected_end_error: impl Fn(usize) -> E,
    ) -> Result<u8, E> {
        let result = self.get(*offset).ok_or(unexpected_end_error(*offset))?;
        *offset += 1;
        Ok(*result)
    }

    fn next_bytes<E>(
        &self,
        length: usize,
        offset: &mut usize,
        unexpected_end_error: impl Fn(usize) -> E,
    ) -> Result<&[u8], E> {
        let result = self
            .get(*offset..*offset + length)
            .ok_or(unexpected_end_error(*offset))?;
        *offset += 1;
        Ok(result)
    }
}

impl<'a> NextBytes<'a> for Box<[u8]> {
    fn next_byte<E>(
        &self,
        offset: &mut usize,
        unexpected_end_error: impl Fn(usize) -> E,
    ) -> Result<u8, E> {
        let result = self.get(*offset).ok_or(unexpected_end_error(*offset))?;
        *offset += 1;
        Ok(*result)
    }

    fn next_bytes<E>(
        &self,
        length: usize,
        offset: &mut usize,
        unexpected_end_error: impl Fn(usize) -> E,
    ) -> Result<&[u8], E> {
        let result = self
            .get(*offset..*offset + length)
            .ok_or(unexpected_end_error(*offset))?;
        *offset += 1;
        Ok(result)
    }
}

impl<'a> NextBytes<'a> for Vec<u8> {
    fn next_byte<E>(
        &self,
        offset: &mut usize,
        unexpected_end_error: impl Fn(usize) -> E,
    ) -> Result<u8, E> {
        let result = self.get(*offset).ok_or(unexpected_end_error(*offset))?;
        *offset += 1;
        Ok(*result)
    }

    fn next_bytes<E>(
        &self,
        length: usize,
        offset: &mut usize,
        unexpected_end_error: impl Fn(usize) -> E,
    ) -> Result<&[u8], E> {
        let result = self
            .get(*offset..*offset + length)
            .ok_or(unexpected_end_error(*offset))?;
        *offset += 1;
        Ok(result)
    }
}
