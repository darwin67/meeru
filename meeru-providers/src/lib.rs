//! Meeru Providers - Email provider adapters
//!
//! This crate contains implementations for various email providers
//! including Gmail, Outlook, and generic IMAP/SMTP servers.

pub mod adapter;
pub mod error;
pub mod generic;
pub mod imap;
pub mod parser;
pub mod smtp;
pub mod traits;

pub use adapter::GenericImapSmtpAdapter;
pub use error::{Error, Result};
pub use generic::{
    AuthMethod, FetchedMessage, GenericAccountConfig, GenericCredentials, GenericMailbox,
    ImapEndpoint, ImapEnvelopeSummary, ImapMessageIdentity, OutgoingMessage, SmtpEndpoint,
    TransportSecurity,
};
pub use parser::{parse_raw_message, ParsedAttachment, ParsedEmailAddress, ParsedMessage};
pub use traits::{EmailProvider, ProviderCapabilities};
