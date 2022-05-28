use ruzzic_common::QuicVersions;
use server::RuzzicServer;
use std::{default::Default, marker::PhantomData, net::SocketAddr};
use tokio::net::UdpSocket;

mod tokio_stream;

pub mod error;
pub mod server;
pub mod simple_app;

pub use self::{error::RuzzicError, error::RuzzicResult, simple_app::SimpleApp};

pub struct Ruzzic<App>
where
    App: AppLayer,
{
    support_versions: QuicVersions,
    socket: UdpSocket,
    _phantom: PhantomData<fn() -> App>,
}

impl<App> Ruzzic<App>
where
    App: AppLayer,
{
    pub async fn send_once(
        &self,
        address: SocketAddr,
        message: App::Message,
    ) -> RuzzicResult<App::Message> {
        let message: App::Message = {
            let mut buf = Vec::new();
            self.socket.recv_from(&mut buf).await?;
            App::Message::from_bytes(&buf)
                .await
                .map_err(App::Error::to_apps)?
        };
        Ok(message)
    }

    pub async fn server(self) -> RuzzicServer<App> {
        RuzzicServer::new(self.support_versions, self.socket).await
    }
}

pub trait AppLayer {
    type Message: AppMessage<Self::Error>;
    type Error: AppError;
}

#[async_trait::async_trait]
pub trait AppMessage<E>: Sized
where
    E: AppError,
{
    async fn to_bytes(&self) -> Result<Vec<u8>, E>;
    async fn from_bytes(buf: &[u8]) -> Result<Self, E>;
}

#[async_trait::async_trait]
pub trait AppError {
    fn to_apps(self) -> RuzzicError;
}

pub struct RuzzicInit<'a, App> {
    pub support_versions: QuicVersions,
    pub self_addr: &'a str,
    pub _phantom: PhantomData<fn() -> App>,
}

impl<App> RuzzicInit<'_, App>
where
    App: AppLayer,
{
    pub async fn init(self) -> RuzzicResult<Ruzzic<App>> {
        let socket = UdpSocket::bind(self.self_addr.parse::<SocketAddr>()?).await?;
        Ok(Ruzzic {
            support_versions: self.support_versions,
            socket,
            _phantom: PhantomData,
        })
    }
}

impl<App> Default for RuzzicInit<'_, App> {
    fn default() -> Self {
        Self {
            support_versions: Vec::new(),
            self_addr: "0.0.0.0:0",
            _phantom: PhantomData,
        }
    }
}
