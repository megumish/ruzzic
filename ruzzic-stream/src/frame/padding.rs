#[cfg(test)]
mod tests {
    use crate::{frame::Frame, ReadBytesTo};
    use std::io::Cursor;

    #[test]
    fn padding_frame() {
        let buf = [0x00];
        let mut input = Cursor::new(buf);
        let actual: Frame = input.read_bytes_to().unwrap();
        let expect = Frame::Padding;
        assert_eq!(actual, expect);
    }
}
