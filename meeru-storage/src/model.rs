//! Storage-layer record types used by the SQLite backend.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    Gmail,
    Outlook,
    Generic,
}

impl ProviderType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gmail => "gmail",
            Self::Outlook => "outlook",
            Self::Generic => "generic",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "gmail" => Ok(Self::Gmail),
            "outlook" => Ok(Self::Outlook),
            "generic" => Ok(Self::Generic),
            other => Err(Error::Other(format!(
                "unknown provider_type value in storage: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountRecord {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub provider_type: ProviderType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewAccount {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub provider_type: ProviderType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnifiedFolderType {
    Inbox,
    Sent,
    Drafts,
    Archive,
    Trash,
    Spam,
    Custom,
}

impl UnifiedFolderType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Sent => "sent",
            Self::Drafts => "drafts",
            Self::Archive => "archive",
            Self::Trash => "trash",
            Self::Spam => "spam",
            Self::Custom => "custom",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "inbox" => Ok(Self::Inbox),
            "sent" => Ok(Self::Sent),
            "drafts" => Ok(Self::Drafts),
            "archive" => Ok(Self::Archive),
            "trash" => Ok(Self::Trash),
            "spam" => Ok(Self::Spam),
            "custom" => Ok(Self::Custom),
            other => Err(Error::Other(format!(
                "unknown folder_type value in storage: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnifiedFolderRecord {
    pub id: Uuid,
    pub name: String,
    pub folder_type: UnifiedFolderType,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewUnifiedFolder {
    pub id: Uuid,
    pub name: String,
    pub folder_type: UnifiedFolderType,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderMappingRecord {
    pub id: Uuid,
    pub unified_folder_id: Uuid,
    pub account_id: Uuid,
    pub provider_folder_id: String,
    pub provider_folder_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewFolderMapping {
    pub id: Uuid,
    pub unified_folder_id: Uuid,
    pub account_id: Uuid,
    pub provider_folder_id: String,
    pub provider_folder_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress {
    pub address: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub provider_id: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub from: Option<EmailAddress>,
    pub to: Vec<EmailAddress>,
    pub date_internal: DateTime<Utc>,
    pub content_file_path: Option<String>,
    pub has_attachments: bool,
    pub attachment_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewEmail {
    pub id: Uuid,
    pub account_id: Uuid,
    pub provider_id: String,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub from: Option<EmailAddress>,
    pub to: Vec<EmailAddress>,
    pub date_internal: DateTime<Utc>,
    pub content_file_path: Option<String>,
    pub has_attachments: bool,
    pub attachment_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewEmailBundle {
    pub email: NewEmail,
    pub folder_ids: Vec<Uuid>,
    pub attachments: Vec<NewAttachment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentRecord {
    pub id: Uuid,
    pub email_id: Uuid,
    pub filename: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub file_path: Option<String>,
    pub file_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewAttachment {
    pub id: Uuid,
    pub email_id: Uuid,
    pub filename: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub file_path: Option<String>,
    pub file_hash: Option<String>,
}
