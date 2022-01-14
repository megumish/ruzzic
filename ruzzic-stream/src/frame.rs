use crate::{read_varint, FromReadBytes, ReadBytesTo};

mod ack;
mod crypto;
mod new_token;
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
    Crypto(crypto::Body),
    NewToken(new_token::Body),
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
            0x06 => {
                let body = input.read_bytes_to()?;
                Frame::Crypto(body)
            }
            0x07 => {
                let body = input.read_bytes_to()?;
                Frame::NewToken(body)
            }
            _ => Frame::Extension(frame_type),
        })
    }
}
