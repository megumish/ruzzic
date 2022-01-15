use bitvec::prelude::*;

use crate::{read_varint, FromReadBytes, ReadBytesTo};

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
    Extension,
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
            0x02 | 0x03 => Frame::Ack(ack::Body::read_bytes(input, frame_type)?),
            0x04 => Frame::ResetStream(input.read_bytes_to()?),
            0x05 => Frame::StopSending(input.read_bytes_to()?),
            0x06 => Frame::Crypto(input.read_bytes_to()?),
            0x07 => Frame::NewToken(input.read_bytes_to()?),
            x if (0x08..0x0f).contains(&x) => {
                let mut flags = bitvec![Msb0, u8; 1];
                flags.store(x);
                Frame::Stream(stream::Body::read_bytes_to(input, &flags[5..])?)
            }
            0x10 => Frame::MaxData(input.read_bytes_to()?),
            0x11 => Frame::MaxStreamData(input.read_bytes_to()?),
            0x12 | 0x13 => Frame::MaxStreams(max_streams::Body::read_bytes_to(input, frame_type)?),
            0x14 => Frame::DataBlocked(input.read_bytes_to()?),
            0x15 => Frame::StreamDataBlocked(input.read_bytes_to()?),
            0x16 | 0x17 => {
                Frame::StreamsBlocked(streams_blocked::Body::read_bytes_to(input, frame_type)?)
            }
            0x18 => Frame::NewConnectionID(input.read_bytes_to()?),
            0x19 => Frame::RetireConnectionID(input.read_bytes_to()?),
            0x1a => Frame::PathChallenge(input.read_bytes_to()?),
            0x1b => Frame::PathResponse(input.read_bytes_to()?),
            0x1c | 0x1d => {
                Frame::ConnectionClose(connection_close::Body::read_bytes_to(input, frame_type)?)
            }
            _ => Frame::Extension(frame_type),
        })
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
            _ => FrameType::Extension,
        }
    }
}
