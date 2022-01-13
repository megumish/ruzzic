#[cfg(test)]
mod tests {
    use std::io::Cursor; 
    use crate::{ReadBytesTo, frame::Frame};

    #[test]
    fn ping_frame() {
        let buf = [0x01];
        let mut input = Cursor::new(buf);
        let actual: Frame = input.read_bytes_to().unwrap();
        let expect = Frame::Ping;
        assert_eq!(actual, expect);

    }
}