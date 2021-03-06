use std::{io::Cursor, pin::Pin};

use bytes::{Buf, BufMut, BytesMut};
use futures_sink::Sink;
use ruzzic_common::{read_bytes_to::FromReadBytes, EndpointType};
use ruzzic_stream::packet::Packet;
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::{
    codec::{BytesCodec, Decoder, Encoder},
    udp::UdpFramed,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("localhost:8080").await?;
    let codec = MeguCodec;
    let mut udpframed = UdpFramed::new(socket, codec);
    let (packet, addr) = udpframed.next().await.unwrap()?;
    println!("{packet:?}");
    {
        let mut udpframed = Pin::new(&mut udpframed);
        udpframed.start_send((packet, addr));
    }

    Ok(())
}

pub struct MeguCodec;

impl Decoder for MeguCodec {
    type Error = anyhow::Error;
    type Item = Packet;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut packet_reader = Cursor::new(src.as_ref());

        let packet = match Packet::from_read_bytes(&mut packet_reader) {
            Ok(p) => p,
            // All errors are considered insufficient packet length.
            // TODO: #10
            Err(e) => return Ok(None),
        };
        src.advance(packet.raw_length());

        //let packet = packet.decrypt(&EndpointType::Server, None);
        todo!();

        Ok(Some(packet))
    }
}

impl Encoder<Packet> for MeguCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, packet: Packet, destination: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}
