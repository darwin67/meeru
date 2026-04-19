//! Email data structures and operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Email {
    pub id: Uuid,
    pub account_id: Uuid,
    pub provider_id: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub from: Option<EmailAddress>,
    pub to: Vec<EmailAddress>,
    pub date: DateTime<Utc>,
    pub content_ref: Option<String>,
    pub content: Option<EmailContent>,
    pub has_attachments: bool,
    pub attachment_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress {
    pub address: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailContent {
    pub text: Option<String>,
    pub html: Option<String>,
}
