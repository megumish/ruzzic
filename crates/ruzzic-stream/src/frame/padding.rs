#[cfg(test)]
mod tests {
    use ruzzic_common::read_bytes_to::ReadBytesTo;
    use std::io::Cursor;

    use crate::frame::Frame;

    #[test]
    fn padding_frame() {
        let buf = [0x00];
        let mut input = Cursor::new(buf);
        let actual: Frame = input.read_bytes_to().unwrap();
        let expect = Frame::Padding;
        assert_eq!(actual, expect);
    }
}
