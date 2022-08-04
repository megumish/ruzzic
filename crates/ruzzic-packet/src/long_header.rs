use std::{
    io::{Cursor, Read},
    marker::PhantomData,
};

use byteorder::ReadBytesExt;
use ruzzic_common::{next_bytes::NextBytes, QuicVersion};

use crate::packet::{Packet, PacketTransformError};

pub struct LongHeaderPacket<'a> {
    first_byte: u8,
    version: QuicVersion,
    destination_connection_id: &'a [u8],
    source_connection_id: &'a [u8],
    type_specific_payload: &'a [u8],
}

impl<'a> LongHeaderPacket<'a> {
    pub fn first_byte(&self) -> u8 {
        self.first_byte
    }

    pub fn version(&self) -> QuicVersion {
        self.version.clone()
    }

    pub fn destination_connection_id(&self) -> &[u8] {
        self.destination_connection_id
    }

    pub fn source_connection_id(&self) -> &[u8] {
        self.source_connection_id
    }

    pub fn type_specific_payload<'b>(&'b self) -> &'b [u8] {
        self.type_specific_payload
    }

    pub(crate) fn packet_number_length(&self) -> usize {
        (self.first_byte & 0b0000_0011) as usize
    }
}

impl<'a> TryFrom<&'a Packet> for LongHeaderPacket<'a> {
    type Error = PacketTransformError<Self>;
    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if is_long(packet) {
            let next_bytes = packet.next_bytes();
            let position = &mut 0;
            let connection_id = &mut || {
                let length = next_bytes.next_byte(position, Self::Error::UnexpectedEnd)? as usize;

                let id = next_bytes.next_bytes(length, position, Self::Error::UnexpectedEnd)?;
                Ok(id)
            };
            let destination_connection_id = connection_id()?;
            let source_connection_id = connection_id()?;
            let type_specific_payload = &next_bytes[*position..];
            Ok(Self {
                first_byte: packet.first_byte(),
                version: packet.version(),
                destination_connection_id,
                source_connection_id,
                type_specific_payload,
            })
        } else {
            Err(Self::Error::NotThisKind)
        }
    }
}

pub fn is_long(packet: &Packet) -> bool {
    (packet.first_byte() & 0b1000_0000) != 0
}

#[derive(thiserror::Error, Debug)]
pub enum LongHeaderPacketTransformError<P> {
    #[error("not this kind")]
    NotThisKind,
    #[error("unexpected end")]
    UnexpectedEnd(usize),
    #[error("this is marker vairant")]
    _TypeMarker(PhantomData<P>),
}
