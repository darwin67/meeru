use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct Account {
    pub id: i64,
    pub display_name: String,
    pub email_address: String,
    pub provider: Option<String>,
    pub primary_protocol: String, // "imap" | "pop3" | "jmap" | "activesync"
}

#[derive(Debug, FromRow, Clone)]
pub struct Folder {
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub full_path: Option<String>,
    pub role: Option<String>,
    pub is_virtual: bool,
    pub parent_id: Option<i64>,
    pub remote_id: Option<String>,
    pub remote_parent_id: Option<String>,
}

#[derive(Debug, FromRow, Clone)]
pub struct Message {
    pub id: i64,
    pub account_id: i64,
    pub external_message_id: Option<String>,
    pub thread_key: Option<String>,
    pub subject: Option<String>,
    pub from_addr: Option<String>,
    pub to_addrs: Option<String>,
    pub cc_addrs: Option<String>,
    pub bcc_addrs: Option<String>,
    pub date: Option<i64>,
    pub size: Option<i64>,
    pub preview_snippet: Option<String>,
    pub body_storage_key: Option<String>,
    pub attachment_manifest: Option<String>,
}

#[derive(Debug, FromRow, Clone)]
pub struct FolderMessage {
    pub folder_id: i64,
    pub message_id: i64,
    pub is_read: bool,
    pub is_flagged: bool,
    pub is_answered: bool,
    pub is_deleted: bool,
    pub raw_flags: Option<String>,
}

// Protocol specific

#[derive(Debug, FromRow, Clone)]
pub struct ImapFolderState {
    pub folder_id: i64,
    pub uid_validity: Option<i64>,
    pub highest_modseq: Option<i64>,
    pub uid_next: Option<i64>,
    pub highest_uid: Option<i64>,
    pub last_sync_ts: Option<i64>,
}

#[derive(Debug, FromRow, Clone)]
pub struct ImapMessageState {
    pub folder_id: i64,
    pub message_id: i64,
    pub uid: i64,
    pub modseq: Option<i64>,
}

#[derive(Debug, FromRow, Clone)]
pub struct Pop3MessageState {
    pub account_id: i64,
    pub uidl: String,
    pub message_id: Option<i64>,
    pub is_deleted: bool,
}

#[derive(Debug, FromRow, Clone)]
pub struct JmapState {
    pub account_id: i64,
    pub mail_state: Option<String>,
    pub msg_state: Option<String>,
}

#[derive(Debug, FromRow, Clone)]
pub struct JmapMessageState {
    pub account_id: i64,
    pub jmap_id: String,
    pub message_id: i64,
}

#[derive(Debug, FromRow, Clone)]
pub struct EasFolderState {
    pub folder_id: i64,
    pub sync_key: Option<String>,
    pub last_sync_ts: Option<i64>,
}

#[derive(Debug, FromRow, Clone)]
pub struct EasMessageState {
    pub folder_id: i64,
    pub server_id: String,
    pub message_id: i64,
}
