use bitvec::prelude::*;

use crate::{
    read_bytes_to::{FromReadBytesWith, ReadBytesTo, ReadBytesToWith},
    read_varint,
};

mod ack;
mod connection_close;
mod crypto;
mod data_blocked;
mod max_data;
mod max_stream_data;
mod max_streams;
mod new_connection_id;
mod new_token;
mod padding;
mod path_challenge;
mod path_response;
mod ping;
mod reset_stream;
mod retire_connection_id;
mod stop_sending;
mod stream;
mod stream_data_blocked;
mod streams_blocked;

#[derive(Debug, PartialEq)]
pub struct Frames(Vec<Frame>);

#[derive(Debug, PartialEq)]
enum Frame {
    Padding,
    Ping,
    Ack(ack::Body),
    ResetStream(reset_stream::Body),
    StopSending(stop_sending::Body),
    Crypto(crypto::Body),
    NewToken(new_token::Body),
    Stream(stream::Body),
    MaxData(max_data::Body),
    MaxStreamData(max_stream_data::Body),
    MaxStreams(max_streams::Body),
    DataBlocked(data_blocked::Body),
    StreamDataBlocked(stream_data_blocked::Body),
    StreamsBlocked(streams_blocked::Body),
    NewConnectionID(new_connection_id::Body),
    RetireConnectionID(retire_connection_id::Body),
    PathChallenge(path_challenge::Body),
    PathResponse(path_response::Body),
    ConnectionClose(connection_close::Body),
    HandshakeDone,
    Extension(u64),
}

#[derive(Debug, PartialEq)]
pub enum FrameType {
    Padding,
    Ping,
    Ack,
    ResetStream,
    StopSending,
    Crypto,
    NewToken,
    Stream,
    MaxData,
    MaxStreamData,
    MaxStreams,
    DataBlocked,
    StreamDataBlocked,
    StreamsBlocked,
    NewConnectionID,
    RetireConnectionID,
    PathChallenge,
    PathResponse,
    ConnectionClose,
    HandshakeDone,
    Extension,
}

impl FromReadBytesWith<()> for Frame {
    fn from_read_bytes_with<T: std::io::Read>(input: &mut T, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let frame_type = read_varint(input)?.to_u64();
        Ok(match frame_type {
            0x00 => Frame::Padding,
            0x01 => Frame::Ping,
            0x02 | 0x03 => Frame::Ack(input.read_bytes_to_with(frame_type)?),
            0x04 => Frame::ResetStream(input.read_bytes_to()?),
            0x05 => Frame::StopSending(input.read_bytes_to()?),
            0x06 => Frame::Crypto(input.read_bytes_to()?),
            0x07 => Frame::NewToken(input.read_bytes_to()?),
            x if (0x08..0x0f).contains(&x) => {
                let mut flags = bitarr![Msb0, u8; 0; 1];
                flags.store(x);
                Frame::Stream(input.read_bytes_to_with(&flags[5..])?)
            }
            0x10 => Frame::MaxData(input.read_bytes_to()?),
            0x11 => Frame::MaxStreamData(input.read_bytes_to()?),
            0x12 | 0x13 => Frame::MaxStreams(input.read_bytes_to_with(frame_type)?),
            0x14 => Frame::DataBlocked(input.read_bytes_to()?),
            0x15 => Frame::StreamDataBlocked(input.read_bytes_to()?),
            0x16 | 0x17 => Frame::StreamsBlocked(input.read_bytes_to_with(frame_type)?),
            0x18 => Frame::NewConnectionID(input.read_bytes_to()?),
            0x19 => Frame::RetireConnectionID(input.read_bytes_to()?),
            0x1a => Frame::PathChallenge(input.read_bytes_to()?),
            0x1b => Frame::PathResponse(input.read_bytes_to()?),
            0x1c | 0x1d => Frame::ConnectionClose(input.read_bytes_to_with(frame_type)?),
            0x1e => Frame::HandshakeDone,
            _ => Frame::Extension(frame_type),
        })
    }
}

impl FromReadBytesWith<()> for Frames {
    fn from_read_bytes_with<R: std::io::Read>(input: &mut R, _: ()) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut frames = Vec::new();
        while let Ok(frame) = input.read_bytes_to() {
            frames.push(frame);
        }
        Ok(Frames(frames))
    }
}

impl FrameType {
    fn from_u64(x: u64) -> Self {
        match x {
            0x00 => FrameType::Padding,
            0x01 => FrameType::Ping,
            0x02 | 0x03 => FrameType::Ack,
            0x04 => FrameType::ResetStream,
            0x05 => FrameType::StopSending,
            0x06 => FrameType::Crypto,
            0x07 => FrameType::NewToken,
            x if (0x08..0x0f).contains(&x) => FrameType::Stream,
            0x10 => FrameType::MaxData,
            0x11 => FrameType::MaxStreamData,
            0x12 | 0x13 => FrameType::MaxStreams,
            0x14 => FrameType::DataBlocked,
            0x15 => FrameType::StreamDataBlocked,
            0x16 | 0x17 => FrameType::StreamsBlocked,
            0x18 => FrameType::NewConnectionID,
            0x19 => FrameType::RetireConnectionID,
            0x1a => FrameType::PathChallenge,
            0x1b => FrameType::PathResponse,
            0x1c | 0x1d => FrameType::ConnectionClose,
            0x1e => FrameType::HandshakeDone,
            _ => FrameType::Extension,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn empty_frames() {
        let buf = [];
        let mut input = Cursor::new(buf);
        let frames: Frames = input.read_bytes_to().unwrap();
        assert_eq!(frames, Frames(Vec::new()));
    }

    #[test]
    fn neqo_server_initial_packet_frames() {
        let buf = [
            2, 0, 0, 0, 0, 6, 0, 64, 90, 2, 0, 0, 86, 3, 3, 219, 98, 183, 101, 225, 209, 143, 84,
            159, 231, 81, 246, 36, 1, 52, 248, 222, 203, 11, 68, 30, 155, 62, 173, 174, 167, 185,
            90, 104, 45, 91, 10, 0, 19, 1, 0, 0, 46, 0, 51, 0, 36, 0, 29, 0, 32, 18, 139, 193, 217,
            226, 59, 133, 108, 95, 30, 210, 203, 91, 196, 57, 52, 155, 5, 36, 50, 96, 211, 110,
            174, 98, 245, 73, 178, 5, 87, 111, 106, 0, 43, 0, 2, 3, 4,
        ];
        let mut input = Cursor::new(buf);
        let frames: Frames = input.read_bytes_to().unwrap();
        eprintln!("{frames:?}");
    }

    #[test]
    fn neqo_nazo_packet_frames() {
        let buf = [
            2, 1, 0, 0, 1, 10, 3, 31, 0, 4, 12, 1, 128, 0, 64, 0, 7, 10, 171, 96, 55, 66, 0, 207,
            77, 135, 85, 177, 9, 242, 171, 7, 200, 52, 89, 85, 192, 179, 82, 10, 7, 1, 2, 10, 11,
            1, 3, 6, 0, 65, 45, 4, 0, 1, 41, 0, 2, 163, 0, 4, 106, 250, 133, 2, 0, 0, 1, 18, 180,
            60, 196, 205, 255, 233, 32, 98, 35, 56, 249, 79, 0, 0, 0, 0, 242, 92, 164, 4, 159, 150,
            187, 61, 37, 178, 15, 31, 39, 209, 171, 73, 0, 208, 192, 144, 113, 135, 244, 106, 71,
            86, 186, 235, 76, 43, 11, 188, 27, 80, 155, 70, 208, 45, 11, 212, 197, 254, 124, 95,
            202, 118, 130, 215, 160, 239, 219, 49, 174, 146, 96, 207, 116, 154, 148, 235, 3, 29,
            233, 171, 205, 174, 197, 179, 120, 205, 118, 237, 190, 26, 150, 149, 131, 189, 170,
            181, 243, 133, 112, 45, 68, 38, 246, 169, 76, 133, 60, 64, 56, 49, 84, 91, 38, 226,
            188, 171, 232, 224, 60, 210, 48, 212, 216, 66, 42, 95, 32, 187, 180, 212, 92, 245, 119,
            27, 251, 243, 133, 68, 210, 244, 157, 93, 193, 197, 107, 21, 122, 63, 88, 144, 28, 212,
            200, 93, 129, 90, 212, 12, 145, 111, 169, 100, 3, 35, 7, 207, 47, 138, 50, 96, 192,
            191, 214, 32, 254, 240, 239, 93, 129, 177, 203, 158, 250, 192, 93, 56, 39, 227, 142, 6,
            146, 239, 208, 219, 116, 237, 240, 50, 61, 168, 46, 16, 137, 217, 45, 110, 20, 66, 167,
            203, 9, 226, 94, 102, 245, 243, 197, 50, 87, 14, 240, 123, 149, 241, 223, 163, 34, 220,
            9, 36, 3, 170, 111, 242, 122, 144, 160, 45, 145, 133, 160, 230, 225, 78, 182, 10, 19,
            152, 199, 112, 46, 90, 178, 179, 169, 138, 92, 131, 166, 220, 171, 86, 206, 234, 249,
            170, 19, 143, 0, 28, 252, 104, 145, 186, 0, 8, 0, 42, 0, 4, 255, 255, 255, 255, 7, 43,
            173, 154, 139, 141, 134, 1, 0, 18, 251, 254, 79, 158, 177, 222, 70, 252, 128, 151, 131,
            245, 118, 123, 123, 204, 130, 70, 66, 98, 193, 216, 24, 51, 124, 147, 151, 57, 216, 80,
            252, 111, 149, 167, 119,
        ];
        let mut input = Cursor::new(buf[..].to_owned());
        let frames: Frames = input.read_bytes_to().unwrap();
        eprintln!("{frames:?}");
    }
}
