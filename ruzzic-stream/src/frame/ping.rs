#[cfg(test)]
mod tests {
    use crate::{frame::Frame, ReadBytesTo};
    use std::io::Cursor;

    #[test]
    fn ping_frame() {
        let buf = [0x01];
        let mut input = Cursor::new(buf);
        let actual: Frame = input.read_bytes_to().unwrap();
        let expect = Frame::Ping;
        assert_eq!(actual, expect);
    }
}
