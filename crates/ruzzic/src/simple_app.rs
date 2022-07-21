mod error;

use crate::{AppLayer, AppMessage};

pub use self::error::{RuzzicSimpleAppError, RuzzicSimpleAppResult};

pub struct SimpleApp;

impl AppLayer for SimpleApp {
    type Message = SimpleAppMessage;
    type Error = RuzzicSimpleAppError;
}

#[derive(Debug)]
pub struct SimpleAppMessage {
    message: String,
}

#[async_trait::async_trait]
impl AppMessage<RuzzicSimpleAppError> for SimpleAppMessage {
    async fn to_bytes(&self) -> RuzzicSimpleAppResult<Vec<u8>> {
        Ok(self.message.as_bytes().to_owned())
    }

    async fn from_bytes(bytes: &[u8]) -> RuzzicSimpleAppResult<Self> {
        Ok(SimpleAppMessage {
            message: String::from_utf8(bytes.to_owned())?,
        })
    }
}
