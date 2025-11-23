// Simplified IMAP client for testing (no TLS)
use anyhow::{Context, Result};
use async_imap::types::{Fetch, Mailbox as ImapMailbox, Name};
use async_imap::Session;
use async_std::net::TcpStream;
use futures::stream::StreamExt;

use super::imap::{EmailAddr, MailboxInfo, MessageData, MessageEnvelope};

pub struct ImapTestClient {
    session: Session<TcpStream>,
}

impl ImapTestClient {
    /// Connect to IMAP server without TLS (for testing)
    pub async fn connect_plain(
        host: &str,
        port: u16,
        email: &str,
        password: &str,
    ) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        let tcp_stream = TcpStream::connect(&addr)
            .await
            .context(format!("Failed to connect to {}", addr))?;

        let client = async_imap::Client::new(tcp_stream);

        // Login
        let session = client
            .login(email, password)
            .await
            .map_err(|e| anyhow::anyhow!("Login failed: {:?}", e))?;

        Ok(Self { session })
    }

    /// List all mailboxes/folders
    pub async fn list_mailboxes(&mut self) -> Result<Vec<MailboxInfo>> {
        let mut mailboxes = self
            .session
            .list(Some(""), Some("*"))
            .await
            .context("Failed to list mailboxes")?;

        let mut result = Vec::new();
        while let Some(mailbox_result) = mailboxes.next().await {
            let mailbox = mailbox_result.context("Failed to parse mailbox")?;
            result.push(MailboxInfo::from_name(&mailbox));
        }

        Ok(result)
    }

    /// Select a mailbox
    pub async fn select_mailbox(&mut self, mailbox_path: &str) -> Result<ImapMailbox> {
        self.session
            .select(mailbox_path)
            .await
            .context(format!("Failed to select mailbox: {}", mailbox_path))
    }

    /// Fetch messages by UID range
    pub async fn fetch_messages(&mut self, uid_range: &str) -> Result<Vec<MessageData>> {
        let fetch_query = "(UID RFC822.SIZE FLAGS ENVELOPE BODY.PEEK[])";
        let mut fetches = self
            .session
            .uid_fetch(uid_range, fetch_query)
            .await
            .context("Failed to fetch messages")?;

        let mut messages = Vec::new();
        while let Some(fetch_result) = fetches.next().await {
            let fetch = fetch_result.context("Failed to parse fetch response")?;
            messages.push(MessageData::from_fetch(&fetch)?);
        }

        Ok(messages)
    }

    /// Mark messages as seen/read
    pub async fn mark_seen(&mut self, uids: &[u32]) -> Result<()> {
        if uids.is_empty() {
            return Ok(());
        }

        let uid_set = uids.iter()
            .map(|uid| uid.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let mut stream = self.session
            .uid_store(&uid_set, "+FLAGS (\\Seen)")
            .await
            .context("Failed to mark messages as seen")?;

        while let Some(_) = stream.next().await {}

        Ok(())
    }

    /// Mark messages as flagged/starred
    pub async fn mark_flagged(&mut self, uids: &[u32]) -> Result<()> {
        if uids.is_empty() {
            return Ok(());
        }

        let uid_set = uids.iter()
            .map(|uid| uid.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let mut stream = self.session
            .uid_store(&uid_set, "+FLAGS (\\Flagged)")
            .await
            .context("Failed to mark messages as flagged")?;

        while let Some(_) = stream.next().await {}

        Ok(())
    }

    /// Mark messages as unseen/unread
    pub async fn mark_unseen(&mut self, uids: &[u32]) -> Result<()> {
        if uids.is_empty() {
            return Ok(());
        }

        let uid_set = uids.iter()
            .map(|uid| uid.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let mut stream = self.session
            .uid_store(&uid_set, "-FLAGS (\\Seen)")
            .await
            .context("Failed to mark messages as unseen")?;

        while let Some(_) = stream.next().await {}

        Ok(())
    }

    /// Mark messages as unflagged/unstarred
    pub async fn mark_unflagged(&mut self, uids: &[u32]) -> Result<()> {
        if uids.is_empty() {
            return Ok(());
        }

        let uid_set = uids.iter()
            .map(|uid| uid.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let mut stream = self.session
            .uid_store(&uid_set, "-FLAGS (\\Flagged)")
            .await
            .context("Failed to mark messages as unflagged")?;

        while let Some(_) = stream.next().await {}

        Ok(())
    }

    /// Logout and close connection
    pub async fn logout(mut self) -> Result<()> {
        self.session
            .logout()
            .await
            .context("Failed to logout")?;
        Ok(())
    }
}
