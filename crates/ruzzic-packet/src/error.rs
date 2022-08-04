use crate::packet::{
    long_header::LongHeaderPacketTransformError, PacketReadError, PacketTransformError,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error when read bytes")]
    PacketReadError(#[from] PacketReadError),
    #[error("error when packet transform to finer")]
    PacketTransformError(#[from] PacketTransformError),
    #[error("error when long packet transform to finer")]
    LongHeaderPacketTransformError(#[from] LongHeaderPacketTransformError),
}
