use std::{io, marker::PhantomData};

use ruzzic_common::QuicVersion;

pub struct Packet {
    first_byte: u8,
    version: QuicVersion,
    next_bytes: Box<[u8]>,
}

impl Packet {
    pub(crate) fn first_byte(&self) -> u8 {
        self.first_byte
    }

    pub(crate) fn version(&self) -> QuicVersion {
        self.version.clone()
    }

    pub(crate) fn next_bytes<'a>(&'a self) -> &'a Box<[u8]> {
        &self.next_bytes
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PacketTransformError<P> {
    #[error("not this kind")]
    NotThisKind,
    #[error("unexpected end")]
    UnexpectedEnd(usize),
    #[error("this is marker vairant")]
    _TypeMarker(PhantomData<P>),
}
