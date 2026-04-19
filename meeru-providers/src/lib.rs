//! Meeru Providers - Email provider adapters
//!
//! This crate contains implementations for various email providers
//! including Gmail, Outlook, and generic IMAP/SMTP servers.

pub mod error;
pub mod generic;
pub mod imap;
pub mod parser;
pub mod smtp;
pub mod traits;

pub use error::{Error, Result};
pub use generic::{
    AuthMethod, GenericAccountConfig, GenericCredentials, GenericMailbox, ImapEndpoint,
    ImapEnvelopeSummary, ImapMessageIdentity, SmtpEndpoint, TransportSecurity,
};
pub use parser::{parse_rfc822_message, ParsedAttachment, ParsedEmailAddress, ParsedMessage};
pub use traits::{EmailProvider, ProviderCapabilities};
