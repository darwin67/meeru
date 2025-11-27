pub mod imap;
pub mod smtp;
pub mod sync;

// Test-only modules (insecure - no TLS!)
// These are always compiled but should NEVER be used in production
#[cfg(any(test, debug_assertions))]
pub mod imap_test;
#[cfg(any(test, debug_assertions))]
pub mod smtp_test;

pub use imap::{ImapClient, MailboxInfo, MessageData, MessageEnvelope};
pub use smtp::{EmailData, SmtpClient};
pub use sync::EmailSyncService;
