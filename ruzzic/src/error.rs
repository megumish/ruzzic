use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuzzicError {
    #[error("invalid address string")]
    InvalidAddress(#[from] std::net::AddrParseError),
    #[error("io error")]
    IOError(#[from] std::io::Error),
    #[error("application layer error")]
    AppError { app_name: String, error: String },
}

pub type RuzzicResult<T> = std::result::Result<T, RuzzicError>;
