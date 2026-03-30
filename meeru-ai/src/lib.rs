//! Meeru AI - AI/ML features for email processing
//!
//! This crate provides AI-powered features including email categorization,
//! priority detection, summarization, and smart compose.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Model loading error: {0}")]
    ModelLoading(String),

    #[error("Inference error: {0}")]
    Inference(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
