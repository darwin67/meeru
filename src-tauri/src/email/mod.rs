pub mod imap;
pub mod smtp;
pub mod sync;

pub use imap::{ImapClient, MailboxInfo, MessageData, MessageEnvelope};
pub use smtp::{SmtpClient, EmailData};
pub use sync::EmailSyncService;
