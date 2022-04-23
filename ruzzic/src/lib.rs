use std::{default::Default, marker::PhantomData, net::SocketAddr};
use tokio::net::UdpSocket;

pub mod error;
pub mod version;

pub use self::{error::RuzzicError, error::RuzzicResult, version::QUICVersion};

pub struct Ruzzic<App>
where
    App: AppLayer,
{
    version: QUICVersion,
    socket: UdpSocket,
    _phantom: PhantomData<App>,
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
    pub version: QUICVersion,
    pub local_addr: &'a str,
    pub _phantom: PhantomData<App>,
}

impl<App> RuzzicInit<'_, App>
where
    App: AppLayer,
{
    pub async fn init(self) -> RuzzicResult<Ruzzic<App>> {
        let socket = UdpSocket::bind(self.local_addr.parse::<SocketAddr>()?).await?;
        Ok(Ruzzic {
            version: self.version,
            socket,
            _phantom: PhantomData,
        })
    }
}

impl<App> Default for RuzzicInit<'_, App> {
    fn default() -> Self {
        Self {
            version: QUICVersion::RFC9000,
            local_addr: "0.0.0.0:0",
            _phantom: PhantomData,
        }
    }
}
