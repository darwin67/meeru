pub mod imap;
pub mod smtp;
pub mod sync;

// Test-only modules (available in tests and when testing feature is enabled)
#[cfg(any(test, feature = "test-utils"))]
pub mod imap_test;
#[cfg(any(test, feature = "test-utils"))]
pub mod smtp_test;

pub use imap::{ImapClient, MailboxInfo, MessageData, MessageEnvelope};
pub use smtp::{SmtpClient, EmailData};
pub use sync::EmailSyncService;
