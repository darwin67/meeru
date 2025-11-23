use anyhow::{Context, Result};
use async_imap::types::{Fetch, Mailbox as ImapMailbox, Name};
use async_imap::Session;
use async_native_tls::{TlsConnector, TlsStream};
use async_std::net::TcpStream;
use futures::stream::StreamExt;

pub struct ImapClient {
    session: Session<TlsStream<TcpStream>>,
}

impl ImapClient {
    /// Connect to IMAP server and authenticate
    pub async fn connect(
        host: &str,
        port: u16,
        email: &str,
        password: &str,
    ) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        let tcp_stream = TcpStream::connect(&addr)
            .await
            .context(format!("Failed to connect to {}", addr))?;

        let tls = TlsConnector::new();
        let tls_stream = tls
            .connect(host, tcp_stream)
            .await
            .context("Failed to establish TLS connection")?;

        let client = async_imap::Client::new(tls_stream);

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

    /// Fetch message UIDs in a mailbox
    pub async fn fetch_uids(&mut self, range: &str) -> Result<Vec<u32>> {
        let mut fetches = self
            .session
            .fetch(range, "UID")
            .await
            .context("Failed to fetch UIDs")?;

        let mut uids = Vec::new();
        while let Some(fetch_result) = fetches.next().await {
            let fetch = fetch_result.context("Failed to parse fetch response")?;
            if let Some(uid) = fetch.uid {
                uids.push(uid);
            }
        }

        Ok(uids)
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

    /// Fetch a single message by UID
    pub async fn fetch_message(&mut self, uid: u32) -> Result<Option<MessageData>> {
        let messages = self.fetch_messages(&uid.to_string()).await?;
        Ok(messages.into_iter().next())
    }

    /// Mark messages as seen/read
    pub async fn mark_seen(&mut self, uids: &[u32]) -> Result<()> {
        if uids.is_empty() {
            return Ok(());
        }

        let uid_set = format_uid_set(uids);
        let mut stream = self.session
            .uid_store(&uid_set, "+FLAGS (\\Seen)")
            .await
            .context("Failed to mark messages as seen")?;

        // Consume the stream
        while let Some(_) = stream.next().await {}

        Ok(())
    }

    /// Mark messages as unseen/unread
    pub async fn mark_unseen(&mut self, uids: &[u32]) -> Result<()> {
        if uids.is_empty() {
            return Ok(());
        }

        let uid_set = format_uid_set(uids);
        let mut stream = self.session
            .uid_store(&uid_set, "-FLAGS (\\Seen)")
            .await
            .context("Failed to mark messages as unseen")?;

        // Consume the stream
        while let Some(_) = stream.next().await {}

        Ok(())
    }

    /// Mark messages as flagged/starred
    pub async fn mark_flagged(&mut self, uids: &[u32]) -> Result<()> {
        if uids.is_empty() {
            return Ok(());
        }

        let uid_set = format_uid_set(uids);
        let mut stream = self.session
            .uid_store(&uid_set, "+FLAGS (\\Flagged)")
            .await
            .context("Failed to mark messages as flagged")?;

        // Consume the stream
        while let Some(_) = stream.next().await {}

        Ok(())
    }

    /// Mark messages as unflagged/unstarred
    pub async fn mark_unflagged(&mut self, uids: &[u32]) -> Result<()> {
        if uids.is_empty() {
            return Ok(());
        }

        let uid_set = format_uid_set(uids);
        let mut stream = self.session
            .uid_store(&uid_set, "-FLAGS (\\Flagged)")
            .await
            .context("Failed to mark messages as unflagged")?;

        // Consume the stream
        while let Some(_) = stream.next().await {}

        Ok(())
    }

    /// Delete messages (mark as deleted and expunge)
    pub async fn delete_messages(&mut self, uids: &[u32]) -> Result<()> {
        if uids.is_empty() {
            return Ok(());
        }

        let uid_set = format_uid_set(uids);
        {
            let mut stream = self.session
                .uid_store(&uid_set, "+FLAGS (\\Deleted)")
                .await
                .context("Failed to mark messages as deleted")?;

            // Consume the stream
            while let Some(_) = stream.next().await {}
        }

        let expunge_stream = self.session
            .expunge()
            .await
            .context("Failed to expunge deleted messages")?;

        // Pin and consume the stream
        futures::pin_mut!(expunge_stream);
        while let Some(_) = expunge_stream.next().await {}

        Ok(())
    }

    /// Move messages to another mailbox
    pub async fn move_messages(&mut self, uids: &[u32], dest_mailbox: &str) -> Result<()> {
        if uids.is_empty() {
            return Ok(());
        }

        let uid_set = format_uid_set(uids);

        // Try MOVE command first (RFC 6851)
        let move_result = self.session
            .uid_mv(&uid_set, dest_mailbox)
            .await;

        match move_result {
            Ok(_) => Ok(()),
            Err(_) => {
                // Fallback to COPY + DELETE
                self.session
                    .uid_copy(&uid_set, dest_mailbox)
                    .await
                    .context("Failed to copy messages")?;

                self.delete_messages(uids).await?;
                Ok(())
            }
        }
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

/// Mailbox information
#[derive(Debug, Clone)]
pub struct MailboxInfo {
    pub name: String,
    pub path: String,
    pub delimiter: Option<String>,
    pub attributes: Vec<String>,
    pub role: Option<String>,
}

impl MailboxInfo {
    pub fn from_name(name: &Name) -> Self {
        let path = name.name().to_string();
        let delimiter = name.delimiter().map(|s| s.to_string());
        let attributes: Vec<String> = name
            .attributes()
            .iter()
            .map(|attr| format!("{:?}", attr))
            .collect();

        // Determine role from attributes or name
        let role = Self::determine_role(&attributes, &path);

        Self {
            name: Self::extract_name(&path, delimiter.as_deref()),
            path,
            delimiter,
            attributes,
            role,
        }
    }

    fn extract_name(path: &str, delimiter: Option<&str>) -> String {
        if let Some(delim) = delimiter {
            path.split(delim).last().unwrap_or(path).to_string()
        } else {
            path.to_string()
        }
    }

    fn determine_role(attributes: &[String], path: &str) -> Option<String> {
        let path_lower = path.to_lowercase();

        // Check attributes first
        for attr in attributes {
            let attr_lower = attr.to_lowercase();
            if attr_lower.contains("inbox") {
                return Some("inbox".to_string());
            } else if attr_lower.contains("sent") {
                return Some("sent".to_string());
            } else if attr_lower.contains("drafts") {
                return Some("drafts".to_string());
            } else if attr_lower.contains("trash") {
                return Some("trash".to_string());
            } else if attr_lower.contains("junk") || attr_lower.contains("spam") {
                return Some("spam".to_string());
            } else if attr_lower.contains("archive") {
                return Some("archive".to_string());
            } else if attr_lower.contains("all") {
                return Some("all".to_string());
            }
        }

        // Fallback to path-based detection
        if path_lower == "inbox" {
            Some("inbox".to_string())
        } else if path_lower.contains("sent") {
            Some("sent".to_string())
        } else if path_lower.contains("draft") {
            Some("drafts".to_string())
        } else if path_lower.contains("trash") || path_lower.contains("deleted") {
            Some("trash".to_string())
        } else if path_lower.contains("junk") || path_lower.contains("spam") {
            Some("spam".to_string())
        } else if path_lower.contains("archive") {
            Some("archive".to_string())
        } else {
            None
        }
    }
}

/// Message data fetched from IMAP
#[derive(Debug, Clone)]
pub struct MessageData {
    pub uid: u32,
    pub size: u32,
    pub flags: Vec<String>,
    pub envelope: Option<MessageEnvelope>,
    pub body: Option<Vec<u8>>,
}

impl MessageData {
    pub fn from_fetch(fetch: &Fetch) -> Result<Self> {
        let uid = fetch.uid.context("Missing UID in fetch response")?;
        let size = fetch.size.unwrap_or(0);

        let flags: Vec<String> = fetch
            .flags()
            .map(|f| format!("{:?}", f))
            .collect();

        let envelope = fetch.envelope().map(|env| MessageEnvelope::from_envelope(env));

        let body = fetch.body().map(|b| b.to_vec());

        Ok(Self {
            uid,
            size,
            flags,
            envelope,
            body,
        })
    }
}

/// Message envelope data
#[derive(Debug, Clone)]
pub struct MessageEnvelope {
    pub date: Option<String>,
    pub subject: Option<String>,
    pub from: Vec<EmailAddr>,
    pub to: Vec<EmailAddr>,
    pub cc: Vec<EmailAddr>,
    pub bcc: Vec<EmailAddr>,
    pub reply_to: Vec<EmailAddr>,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
}

impl MessageEnvelope {
    fn from_envelope(env: &imap_proto::Envelope) -> Self {
        Self {
            date: env.date.as_ref().map(|d| String::from_utf8_lossy(d).to_string()),
            subject: env.subject.as_ref().map(|s| String::from_utf8_lossy(s).to_string()),
            from: env.from.as_ref().map(|v| Self::parse_addresses(v)).unwrap_or_default(),
            to: env.to.as_ref().map(|v| Self::parse_addresses(v)).unwrap_or_default(),
            cc: env.cc.as_ref().map(|v| Self::parse_addresses(v)).unwrap_or_default(),
            bcc: env.bcc.as_ref().map(|v| Self::parse_addresses(v)).unwrap_or_default(),
            reply_to: env.reply_to.as_ref().map(|v| Self::parse_addresses(v)).unwrap_or_default(),
            message_id: env.message_id.as_ref().map(|m| String::from_utf8_lossy(m).to_string()),
            in_reply_to: env.in_reply_to.as_ref().map(|i| String::from_utf8_lossy(i).to_string()),
        }
    }

    fn parse_addresses(addrs: &Vec<imap_proto::Address>) -> Vec<EmailAddr> {
        addrs
            .iter()
            .filter_map(|addr| {
                let mailbox = addr.mailbox.as_ref()?;
                let host = addr.host.as_ref()?;
                let email = format!(
                    "{}@{}",
                    String::from_utf8_lossy(mailbox),
                    String::from_utf8_lossy(host)
                );
                let name = addr.name.as_ref().map(|n| String::from_utf8_lossy(n).to_string());
                Some(EmailAddr { email, name })
            })
            .collect()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmailAddr {
    pub email: String,
    pub name: Option<String>,
}

/// Format a list of UIDs into a comma-separated string
fn format_uid_set(uids: &[u32]) -> String {
    uids.iter()
        .map(|uid| uid.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
