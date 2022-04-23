mod error;

use ruzzic::{AppLayer, AppMessage};
use std::net::SocketAddr;
use url::Url;

pub use self::error::{RuzzicHttp3Error, RuzzicHttp3Result};

pub struct GetRequest {
    url: Url,
}

impl GetRequest {
    pub async fn address(&self) -> SocketAddr {
        let host = self.url.host_str().unwrap();
        let port = self.url.port_or_known_default().unwrap();
        let address = format!("{}:{}", host, port).parse::<SocketAddr>().unwrap();
        address
    }

    pub async fn to_message(&self) -> Http3AppMessage {
        Http3AppMessage::GetRequest(GetRequestMessage {
            url: self.url.clone(),
        })
    }
}

#[derive(Default)]
pub struct GetRequestInit<'a> {
    pub url: &'a str,
}

impl GetRequestInit<'_> {
    pub async fn init(self) -> RuzzicHttp3Result<GetRequest> {
        Ok(GetRequest {
            url: self.url.parse()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GetRequestMessage {
    url: Url,
}

pub struct Http3App;

impl AppLayer for Http3App {
    type Message = Http3AppMessage;
    type Error = RuzzicHttp3Error;
}

#[derive(Debug)]
pub enum Http3AppMessage {
    GetRequest(GetRequestMessage),
    Response(ResponseMessage),
}

#[async_trait::async_trait]
impl AppMessage<RuzzicHttp3Error> for Http3AppMessage {
    async fn to_bytes(&self) -> RuzzicHttp3Result<Vec<u8>> {
        match self {
            Http3AppMessage::GetRequest(message) => {
                Ok(message.url.to_string().as_bytes().to_owned())
            }
            _ => unimplemented!(),
        }
    }

    async fn from_bytes(_: &[u8]) -> RuzzicHttp3Result<Self> {
        Ok(Http3AppMessage::Response(ResponseMessage))
    }
}

#[derive(Debug, Clone)]
pub struct ResponseMessage;

impl Http3AppMessage {
    pub fn to_response(&self) -> RuzzicHttp3Result<ResponseMessage> {
        match self {
            Http3AppMessage::Response(response) => Ok(response.clone()),
            _ => Err(RuzzicHttp3Error::InvalidMessageConversion(format!(
                "expected: Response, but actual: {self:?}"
            ))),
        }
    }
}
