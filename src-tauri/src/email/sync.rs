use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::accounts::AccountManager;
use crate::db::models::{Account, Mailbox};

// Use production IMAP client in release builds, test client in debug builds
#[cfg(not(debug_assertions))]
use crate::email::imap::{ImapClient, MailboxInfo, MessageData};
#[cfg(not(debug_assertions))]
type ActiveImapClient = ImapClient;

#[cfg(debug_assertions)]
use crate::email::imap::{MailboxInfo, MessageData};
#[cfg(debug_assertions)]
use crate::email::imap_test::ImapTestClient;
#[cfg(debug_assertions)]
type ActiveImapClient = ImapTestClient;

/// Email synchronization service
pub struct EmailSyncService {
    pool: SqlitePool,
    account_manager: AccountManager,
}

impl EmailSyncService {
    pub fn new(pool: SqlitePool) -> Self {
        let account_manager = AccountManager::new(pool.clone());
        Self {
            pool,
            account_manager,
        }
    }

    /// Sync all mailboxes for an account
    pub async fn sync_account(&self, account_id: &str) -> Result<SyncResult> {
        let account = self
            .account_manager
            .get_account(account_id)
            .await?
            .context("Account not found")?;

        let password = self
            .account_manager
            .get_password(account_id)
            .context("Failed to retrieve password")?;

        // Connect to IMAP (TLS in production, plain in debug/test mode)
        #[cfg(not(debug_assertions))]
        let mut imap_client = ActiveImapClient::connect(
            &account.imap_host,
            account.imap_port as u16,
            &account.email,
            &password,
        )
        .await
        .context("Failed to connect to IMAP server")?;

        #[cfg(debug_assertions)]
        let mut imap_client = ActiveImapClient::connect_plain(
            &account.imap_host,
            account.imap_port as u16,
            &account.email,
            &password,
        )
        .await
        .context("Failed to connect to IMAP server")?;

        // List and sync mailboxes
        let mailbox_infos = imap_client
            .list_mailboxes()
            .await
            .context("Failed to list mailboxes")?;

        let mut total_synced = 0;
        let mut total_new = 0;

        for mailbox_info in mailbox_infos {
            let mailbox = self
                .sync_mailbox_metadata(&account, &mailbox_info)
                .await?;

            let (synced, new) = self
                .sync_mailbox_messages(&mut imap_client, &account, &mailbox)
                .await?;

            total_synced += synced;
            total_new += new;
        }

        // Update last sync time
        self.account_manager
            .update_last_sync(account_id)
            .await?;

        // Logout
        imap_client.logout().await?;

        Ok(SyncResult {
            total_messages: total_synced,
            new_messages: total_new,
        })
    }

    /// Sync mailbox metadata (create or update mailbox record)
    async fn sync_mailbox_metadata(
        &self,
        account: &Account,
        info: &MailboxInfo,
    ) -> Result<Mailbox> {
        // Check if mailbox exists
        let existing: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM mailboxes WHERE account_id = ? AND path = ?",
        )
        .bind(&account.id)
        .bind(&info.path)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id,)) = existing {
            // Update existing mailbox
            sqlx::query(
                r#"
                UPDATE mailboxes
                SET name = ?, delimiter = ?, flags = ?, role = ?, updated_at = ?
                WHERE id = ?
                "#,
            )
            .bind(&info.name)
            .bind(&info.delimiter)
            .bind(serde_json::to_string(&info.attributes)?)
            .bind(&info.role)
            .bind(Utc::now())
            .bind(&id)
            .execute(&self.pool)
            .await?;

            // Fetch and return updated mailbox
            let mailbox: Mailbox = sqlx::query_as("SELECT * FROM mailboxes WHERE id = ?")
                .bind(&id)
                .fetch_one(&self.pool)
                .await?;

            Ok(mailbox)
        } else {
            // Create new mailbox
            let mailbox = Mailbox::new(account.id.clone(), info.name.clone(), info.path.clone());

            sqlx::query(
                r#"
                INSERT INTO mailboxes (id, account_id, name, path, delimiter, flags, role, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&mailbox.id)
            .bind(&mailbox.account_id)
            .bind(&mailbox.name)
            .bind(&mailbox.path)
            .bind(&info.delimiter)
            .bind(serde_json::to_string(&info.attributes)?)
            .bind(&info.role)
            .bind(mailbox.created_at)
            .bind(mailbox.updated_at)
            .execute(&self.pool)
            .await?;

            Ok(mailbox)
        }
    }

    /// Sync messages in a mailbox
    async fn sync_mailbox_messages(
        &self,
        imap_client: &mut ActiveImapClient,
        account: &Account,
        mailbox: &Mailbox,
    ) -> Result<(u32, u32)> {
        // Select the mailbox
        let selected = imap_client
            .select_mailbox(&mailbox.path)
            .await
            .context(format!("Failed to select mailbox: {}", mailbox.path))?;

        let exists = selected.exists;
        if exists == 0 {
            return Ok((0, 0));
        }

        // Get existing UIDs from database
        let existing_uids: Vec<(i64,)> = sqlx::query_as(
            "SELECT uid FROM emails WHERE account_id = ? AND mailbox_id = ?",
        )
        .bind(&account.id)
        .bind(&mailbox.id)
        .fetch_all(&self.pool)
        .await?;

        let existing_uid_set: std::collections::HashSet<u32> = existing_uids
            .into_iter()
            .map(|(uid,)| uid as u32)
            .collect();

        // Fetch all UIDs from server
        let server_uids = imap_client
            .fetch_uids(&format!("1:{}", exists))
            .await?;

        // Determine new UIDs to fetch
        let new_uids: Vec<u32> = server_uids
            .into_iter()
            .filter(|uid| !existing_uid_set.contains(uid))
            .collect();

        let new_count = new_uids.len() as u32;

        if new_count == 0 {
            return Ok((exists, 0));
        }

        // Fetch new messages in batches
        const BATCH_SIZE: usize = 50;
        for chunk in new_uids.chunks(BATCH_SIZE) {
            let uid_range = format!(
                "{}:{}",
                chunk.first().unwrap(),
                chunk.last().unwrap()
            );

            let messages = imap_client.fetch_messages(&uid_range).await?;

            for message in messages {
                self.store_message(account, mailbox, message).await?;
            }
        }

        Ok((exists, new_count))
    }

    /// Store a message in the database
    async fn store_message(
        &self,
        account: &Account,
        mailbox: &Mailbox,
        message: MessageData,
    ) -> Result<()> {
        // Parse the message body if available
        let (body_text, body_html, from_address, from_name, to_addresses, subject, date, message_id) =
            if let Some(body_bytes) = message.body {
                Self::parse_message_body(&body_bytes)?
            } else {
                // Use envelope data as fallback
                if let Some(envelope) = message.envelope {
                    let from = envelope.from.first();
                    let from_address = from.map(|f| f.email.clone()).unwrap_or_default();
                    let from_name = from.and_then(|f| f.name.clone());

                    let to_addresses = serde_json::to_string(&envelope.to)?;
                    let subject = envelope.subject.clone();
                    let message_id = envelope.message_id.unwrap_or_else(|| Uuid::new_v4().to_string());

                    (
                        None,
                        None,
                        from_address,
                        from_name,
                        to_addresses,
                        subject,
                        envelope.date.unwrap_or_else(|| Utc::now().to_rfc3339()),
                        message_id,
                    )
                } else {
                    return Ok(()); // Skip messages without envelope
                }
            };

        // Parse date first (needed for thread timestamps)
        let parsed_date = chrono::DateTime::parse_from_rfc2822(&date)
            .or_else(|_| chrono::DateTime::parse_from_rfc3339(&date))
            .unwrap_or_else(|_| Utc::now().into())
            .with_timezone(&Utc);

        // Create thread ID (simplified - just use message_id for now)
        let thread_id = Uuid::new_v4().to_string();

        // Create thread entry first (to satisfy foreign key constraint)
        let participants_json = serde_json::to_string(&vec![from_address.as_str()])?;
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO threads (
                id, account_id, subject, participants,
                first_message_at, last_message_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&thread_id)
        .bind(&account.id)
        .bind(&subject)
        .bind(&participants_json)
        .bind(parsed_date)
        .bind(parsed_date)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        // Determine if message is unread
        let is_unread = !message.flags.iter().any(|f| f.contains("Seen"));
        let is_starred = message.flags.iter().any(|f| f.contains("Flagged"));

        // Insert email
        let email_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO emails (
                id, thread_id, account_id, mailbox_id, uid, message_id,
                subject, from_address, from_name, to_addresses,
                date, received_at, size, flags, is_unread, is_starred,
                body_text, body_html, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&email_id)
        .bind(&thread_id)
        .bind(&account.id)
        .bind(&mailbox.id)
        .bind(message.uid as i64)
        .bind(&message_id)
        .bind(&subject)
        .bind(&from_address)
        .bind(&from_name)
        .bind(&to_addresses)
        .bind(parsed_date)
        .bind(Utc::now())
        .bind(message.size as i64)
        .bind(serde_json::to_string(&message.flags)?)
        .bind(is_unread)
        .bind(is_starred)
        .bind(&body_text)
        .bind(&body_html)
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Parse message body (simplified version - in production you'd use mail-parser or similar)
    fn parse_message_body(
        body: &[u8],
    ) -> Result<(
        Option<String>,
        Option<String>,
        String,
        Option<String>,
        String,
        Option<String>,
        String,
        String,
    )> {
        // This is a simplified parser - in production, use a proper email parsing library
        let body_str = String::from_utf8_lossy(body);

        // Extract basic headers (very simplified)
        let mut from_address = String::new();
        let from_name = None;
        let mut to_addresses = String::from("[]");
        let mut subject = None;
        let mut date = Utc::now().to_rfc2822();
        let mut message_id = Uuid::new_v4().to_string();

        for line in body_str.lines() {
            if line.to_lowercase().starts_with("from:") {
                from_address = line[5..].trim().to_string();
            } else if line.to_lowercase().starts_with("to:") {
                to_addresses = serde_json::to_string(&vec![line[3..].trim()])?;
            } else if line.to_lowercase().starts_with("subject:") {
                subject = Some(line[8..].trim().to_string());
            } else if line.to_lowercase().starts_with("date:") {
                date = line[5..].trim().to_string();
            } else if line.to_lowercase().starts_with("message-id:") {
                message_id = line[11..].trim().to_string();
            }
        }

        Ok((
            Some(body_str.to_string()),
            None,
            from_address,
            from_name,
            to_addresses,
            subject,
            date,
            message_id,
        ))
    }
}

#[derive(Debug)]
pub struct SyncResult {
    pub total_messages: u32,
    pub new_messages: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::db::models::AuthType;
    use tempfile::TempDir;

    async fn create_test_service() -> (EmailSyncService, TempDir, AccountManager) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path).await.unwrap();
        let service = EmailSyncService::new(db.pool().clone());
        let account_manager = AccountManager::new(db.pool().clone());
        (service, temp_dir, account_manager)
    }

    #[tokio::test]
    async fn test_sync_service_creation() {
        let (service, _temp_dir, _account_manager) = create_test_service().await;
        // Service should be created successfully
    }
}
