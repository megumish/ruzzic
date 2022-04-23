use ruzzic::{AppError, RuzzicError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuzzicHttp3Error {
    #[error("invalid url")]
    InvalidUrl(#[from] url::ParseError),
    #[error("invalid message conversion")]
    InvalidMessageConversion(String),
}

pub type RuzzicHttp3Result<T> = std::result::Result<T, RuzzicHttp3Error>;

impl AppError for RuzzicHttp3Error {
    fn to_apps(self) -> RuzzicError {
        RuzzicError::AppError {
            app_name: "ruzzic-http3".to_owned(),
            error: self.to_string(),
        }
    }
}
