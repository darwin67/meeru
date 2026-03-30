//! Meeru Providers - Email provider adapters
//!
//! This crate contains implementations for various email providers
//! including Gmail, Outlook, and generic IMAP/SMTP servers.

pub mod error;
pub mod imap;
pub mod smtp;
pub mod traits;

pub use error::{Error, Result};
pub use traits::{EmailProvider, ProviderCapabilities};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
