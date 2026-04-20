//! Provider error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Timeout error")]
    Timeout,

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
