use std::marker::PhantomData;

use ruzzic_common::QuicVersion;
use tokio_stream::Stream;

use crate::{tokio_stream::RuzzicTokioStream, AppLayer};

pub struct RuzzicServer<App>
where
    App: AppLayer,
{
    quic_stream: RuzzicTokioStream,
    _phantom: PhantomData<App>,
}

impl<App> RuzzicServer<App>
where
    App: AppLayer,
{
    pub(crate) async fn new(
        support_versions: Vec<QuicVersion>,
        socket: tokio::net::UdpSocket,
    ) -> Self {
        let quic_stream = RuzzicTokioStream::new(support_versions, socket);
        Self {
            quic_stream,
            _phantom: PhantomData,
        }
    }
}

impl<App> Stream for RuzzicServer<App>
where
    App: AppLayer,
{
    type Item = App::Message;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        todo!()
    }
}
