//! Error handling for meeru-core

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Storage error: {0}")]
    Storage(#[from] meeru_storage::Error),

    #[error("Provider error: {0}")]
    Provider(#[from] meeru_providers::Error),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Email not found: {0}")]
    EmailNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Sync error: {0}")]
    SyncError(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
