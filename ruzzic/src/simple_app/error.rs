use crate::{AppError, RuzzicError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuzzicSimpleAppError {
    #[error("invalid utf-8 message")]
    InvalidUtf8Message(#[from] std::string::FromUtf8Error),
}

pub type RuzzicSimpleAppResult<T> = std::result::Result<T, RuzzicSimpleAppError>;

impl AppError for RuzzicSimpleAppError {
    fn to_apps(self) -> RuzzicError {
        RuzzicError::AppError {
            app_name: "ruzzic-simple-app".to_owned(),
            error: self.to_string(),
        }
    }
}
