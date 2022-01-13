use crate::{read_varint, FromReadBytes, ReadBytesTo};

mod ack;
mod padding;
mod ping;
mod reset_stream;
mod stop_sending;

#[derive(Debug, PartialEq)]
enum Frame {
    Padding,
    Ping,
    Ack(ack::Body),
    ResetStream(reset_stream::Body),
    StopSending(stop_sending::Body),
    Extension(u64),
}

impl FromReadBytes for Frame {
    fn from_read_bytes<T: std::io::Read>(input: &mut T) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let frame_type = read_varint(input)?.to_u64();
        Ok(match frame_type {
            0x00 => Frame::Padding,
            0x01 => Frame::Ping,
            0x02 | 0x03 => {
                let body = ack::Body::read_bytes(input, frame_type)?;
                Frame::Ack(body)
            }
            0x04 => {
                let body = input.read_bytes_to()?;
                Frame::ResetStream(body)
            }
            0x05 => {
                let body = input.read_bytes_to()?;
                Frame::StopSending(body)
            }
            _ => Frame::Extension(frame_type),
        })
    }
}
