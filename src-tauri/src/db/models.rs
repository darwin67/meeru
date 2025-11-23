use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: String, // UUID as string
    pub email: String,
    pub name: Option<String>,
    pub provider: String,
    pub imap_host: String,
    pub imap_port: i64,
    pub smtp_host: String,
    pub smtp_port: i64,
    pub auth_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_sync_at: Option<DateTime<Utc>>,
}

impl Account {
    pub fn new(
        email: String,
        name: Option<String>,
        provider: String,
        imap_host: String,
        imap_port: u16,
        smtp_host: String,
        smtp_port: u16,
        auth_type: AuthType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            email,
            name,
            provider,
            imap_host,
            imap_port: imap_port as i64,
            smtp_host,
            smtp_port: smtp_port as i64,
            auth_type: auth_type.to_string(),
            created_at: now,
            updated_at: now,
            last_sync_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    Password,
    OAuth2,
}

impl AuthType {
    pub fn to_string(&self) -> String {
        match self {
            AuthType::Password => "password".to_string(),
            AuthType::OAuth2 => "oauth2".to_string(),
        }
    }
}

impl std::str::FromStr for AuthType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "password" => Ok(AuthType::Password),
            "oauth2" => Ok(AuthType::OAuth2),
            _ => Err(anyhow::anyhow!("Invalid auth type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Mailbox {
    pub id: String, // UUID as string
    pub account_id: String,
    pub name: String,
    pub path: String,
    pub delimiter: Option<String>,
    pub flags: Option<String>, // JSON
    pub role: Option<String>,
    pub uidvalidity: Option<i64>,
    pub uidnext: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Mailbox {
    pub fn new(account_id: String, name: String, path: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            account_id,
            name,
            path,
            delimiter: None,
            flags: None,
            role: None,
            uidvalidity: None,
            uidnext: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Thread {
    pub id: String, // UUID as string
    pub account_id: String,
    pub subject: String,
    pub participants: String, // JSON
    pub snippet: Option<String>,
    pub message_count: i64,
    pub has_attachments: bool,
    pub is_unread: bool,
    pub is_starred: bool,
    pub is_important: bool,
    pub category: Option<String>,
    pub labels: Option<String>, // JSON
    pub first_message_at: DateTime<Utc>,
    pub last_message_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Email {
    pub id: String, // UUID as string
    pub thread_id: String,
    pub account_id: String,
    pub mailbox_id: String,
    pub uid: i64,
    pub message_id: String,
    pub in_reply_to: Option<String>,
    pub email_references: Option<String>, // JSON
    pub subject: Option<String>,
    pub from_address: String,
    pub from_name: Option<String>,
    pub to_addresses: String, // JSON
    pub cc_addresses: Option<String>, // JSON
    pub bcc_addresses: Option<String>, // JSON
    pub reply_to: Option<String>, // JSON
    pub date: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
    pub size: i64,
    pub flags: Option<String>, // JSON
    pub is_unread: bool,
    pub is_starred: bool,
    pub is_draft: bool,
    pub has_attachments: bool,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub snippet: Option<String>,
    pub headers: Option<String>, // JSON
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Attachment {
    pub id: String, // UUID as string
    pub email_id: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub content_id: Option<String>,
    pub is_inline: bool,
    pub data: Option<Vec<u8>>,
    pub local_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Contact {
    pub id: String, // UUID as string
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_vip: bool,
    pub email_count: i64,
    pub last_emailed_at: Option<DateTime<Utc>>,
    pub metadata: Option<String>, // JSON
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SyncQueueItem {
    pub id: String, // UUID as string
    pub account_id: String,
    pub operation: String,
    pub payload: String, // JSON
    pub status: String,
    pub retry_count: i64,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Email address with name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    pub email: String,
    pub name: Option<String>,
}

impl EmailAddress {
    pub fn new(email: String, name: Option<String>) -> Self {
        Self { email, name }
    }
}
