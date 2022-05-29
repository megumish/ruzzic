use std::{io::Cursor, net::SocketAddr, pin::Pin};

use bytes::{Buf, BytesMut};
use ruzzic_common::{read_bytes_to::FromReadBytes, EndpointType, QuicVersion, QuicVersions};
use ruzzic_stream::packet::Packet;
use tokio_stream::Stream;
use tokio_util::{codec::Decoder, udp::UdpFramed};

pub struct RuzzicTokioStream {
    udp_stream: UdpFramed<RuzzicTokioCodec>,
}

impl RuzzicTokioStream {
    pub(crate) fn new(support_versions: Vec<QuicVersion>, socket: tokio::net::UdpSocket) -> Self {
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
        let udp_stream = self.get_mut();
        Pin::new(udp_stream).poll_next(cx)
    }
}

impl Decoder for RuzzicTokioCodec {
    type Item = Vec<u8>;

    type Error = RuzzicTokioCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut packet_reader = Cursor::new(src.as_ref());

        let packet = match Packet::from_read_bytes(&mut packet_reader) {
            Ok(p) => p,
            // All errors are considered insufficient packet length.
            // TODO: #10
            Err(e) => return Ok(None),
        };
        src.advance(packet.raw_length());

        match packet.version().to_u32() {
            0x1 => {
                if self.support_versions.contains(&QuicVersion::Rfc9000) {
                    QuicVersion::Rfc9000
                } else {
                    return Ok(None);
                }
            }
            _ => {
                return Ok(None);
            }
        };

        let packet = packet.decrypt(&EndpointType::Server, None);

        Ok(None)
    }
}
