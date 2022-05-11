use std::{net::SocketAddr, pin::Pin};

use bytes::BytesMut;
use tokio_stream::Stream;
use tokio_util::{codec::Decoder, udp::UdpFramed};

use crate::version::QuicVersions;

pub struct RuzzicTokioStream {
    udp_stream: UdpFramed<RuzzicTokioCodec>,
}

impl RuzzicTokioStream {
    pub(crate) fn new(
        support_versions: Vec<crate::QuicVersion>,
        socket: tokio::net::UdpSocket,
    ) -> Self {
        Self {
            udp_stream: UdpFramed::new(socket, RuzzicTokioCodec::new(support_versions)),
        }
    }
}

pub struct RuzzicTokioCodec {
    support_versions: QuicVersions,
}
impl RuzzicTokioCodec {
    fn new(support_versions: QuicVersions) -> Self {
        Self { support_versions }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RuzzicTokioCodecError {
    #[error("io error")]
    IOError(#[from] std::io::Error),
}

impl Stream for RuzzicTokioStream {
    type Item = Result<
        (<RuzzicTokioCodec as Decoder>::Item, SocketAddr),
        <RuzzicTokioCodec as Decoder>::Error,
    >;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        Pin::new(&mut self.udp_stream).poll_next(cx)
    }
}

impl Decoder for RuzzicTokioCodec {
    type Item = Vec<u8>;

    type Error = RuzzicTokioCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
      
    }
}
