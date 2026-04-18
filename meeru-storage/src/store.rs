//! Storage traits and SQLite-backed implementations.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    model::{
        AccountRecord, AttachmentRecord, EmailAddress, EmailRecord, FolderMappingRecord,
        NewAccount, NewAttachment, NewEmail, NewFolderMapping, NewUnifiedFolder, ProviderType,
        UnifiedFolderRecord, UnifiedFolderType,
    },
    Error, Result, Storage,
};

#[async_trait]
pub trait AccountStore {
    async fn create_account(&self, account: NewAccount) -> Result<AccountRecord>;
    async fn get_account(&self, account_id: Uuid) -> Result<AccountRecord>;
    async fn list_accounts(&self) -> Result<Vec<AccountRecord>>;
    async fn delete_account(&self, account_id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait FolderStore {
    async fn create_unified_folder(&self, folder: NewUnifiedFolder) -> Result<UnifiedFolderRecord>;
    async fn list_unified_folders(&self) -> Result<Vec<UnifiedFolderRecord>>;
    async fn create_folder_mapping(&self, mapping: NewFolderMapping)
        -> Result<FolderMappingRecord>;
    async fn list_folder_mappings_for_account(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<FolderMappingRecord>>;
    async fn assign_email_to_folder(&self, email_id: Uuid, folder_id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait EmailStore {
    async fn insert_email(&self, email: NewEmail) -> Result<EmailRecord>;
    async fn get_email(&self, email_id: Uuid) -> Result<EmailRecord>;
    async fn list_emails_for_account(
        &self,
        account_id: Uuid,
        limit: usize,
    ) -> Result<Vec<EmailRecord>>;
    async fn list_emails_in_folder(
        &self,
        folder_id: Uuid,
        limit: usize,
    ) -> Result<Vec<EmailRecord>>;
}

#[async_trait]
pub trait AttachmentStore {
    async fn insert_attachment(&self, attachment: NewAttachment) -> Result<AttachmentRecord>;
    async fn list_attachments_for_email(&self, email_id: Uuid) -> Result<Vec<AttachmentRecord>>;
}

#[async_trait]
impl AccountStore for Storage {
    async fn create_account(&self, account: NewAccount) -> Result<AccountRecord> {
        sqlx::query(
            r#"
INSERT INTO accounts (id, email, display_name, provider_type)
VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(account.id)
        .bind(&account.email)
        .bind(&account.display_name)
        .bind(account.provider_type.as_str())
        .execute(self.pool())
        .await?;

        self.get_account(account.id).await
    }

    async fn get_account(&self, account_id: Uuid) -> Result<AccountRecord> {
        let row = sqlx::query(
            r#"
SELECT id, email, display_name, provider_type
FROM accounts
WHERE id = ?
            "#,
        )
        .bind(account_id)
        .fetch_optional(self.pool())
        .await?;

        match row {
            Some(row) => account_from_row(row),
            None => Err(Error::NotFound(format!("account {account_id}"))),
        }
    }

    async fn list_accounts(&self) -> Result<Vec<AccountRecord>> {
        let rows = sqlx::query(
            r#"
SELECT id, email, display_name, provider_type
FROM accounts
ORDER BY email ASC
            "#,
        )
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(account_from_row).collect()
    }

    async fn delete_account(&self, account_id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(account_id)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound(format!("account {account_id}")));
        }

        Ok(())
    }
}

#[async_trait]
impl FolderStore for Storage {
    async fn create_unified_folder(&self, folder: NewUnifiedFolder) -> Result<UnifiedFolderRecord> {
        sqlx::query(
            r#"
INSERT INTO unified_folders (id, name, folder_type, parent_id)
VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(folder.id)
        .bind(&folder.name)
        .bind(folder.folder_type.as_str())
        .bind(folder.parent_id)
        .execute(self.pool())
        .await?;

        let rows = self.list_unified_folders().await?;
        rows.into_iter()
            .find(|record| record.id == folder.id)
            .ok_or_else(|| Error::NotFound(format!("folder {}", folder.id)))
    }

    async fn list_unified_folders(&self) -> Result<Vec<UnifiedFolderRecord>> {
        let rows = sqlx::query(
            r#"
SELECT id, name, folder_type, parent_id
FROM unified_folders
ORDER BY sort_order ASC, name ASC
            "#,
        )
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(folder_from_row).collect()
    }

    async fn create_folder_mapping(
        &self,
        mapping: NewFolderMapping,
    ) -> Result<FolderMappingRecord> {
        sqlx::query(
            r#"
INSERT INTO folder_mappings (
    id,
    unified_folder_id,
    account_id,
    provider_folder_id,
    provider_folder_name
)
VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(mapping.id)
        .bind(mapping.unified_folder_id)
        .bind(mapping.account_id)
        .bind(&mapping.provider_folder_id)
        .bind(&mapping.provider_folder_name)
        .execute(self.pool())
        .await?;

        let rows = self
            .list_folder_mappings_for_account(mapping.account_id)
            .await?;
        rows.into_iter()
            .find(|record| record.id == mapping.id)
            .ok_or_else(|| Error::NotFound(format!("folder mapping {}", mapping.id)))
    }

    async fn list_folder_mappings_for_account(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<FolderMappingRecord>> {
        let rows = sqlx::query(
            r#"
SELECT id, unified_folder_id, account_id, provider_folder_id, provider_folder_name
FROM folder_mappings
WHERE account_id = ?
ORDER BY provider_folder_id ASC
            "#,
        )
        .bind(account_id)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(folder_mapping_from_row).collect()
    }

    async fn assign_email_to_folder(&self, email_id: Uuid, folder_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
INSERT OR IGNORE INTO email_folders (email_id, unified_folder_id)
VALUES (?, ?)
            "#,
        )
        .bind(email_id)
        .bind(folder_id)
        .execute(self.pool())
        .await?;

        Ok(())
    }
}

#[async_trait]
impl EmailStore for Storage {
    async fn insert_email(&self, email: NewEmail) -> Result<EmailRecord> {
        sqlx::query(
            r#"
INSERT INTO emails (
    id,
    account_id,
    provider_id,
    message_id,
    subject,
    from_address,
    from_name,
    to_addresses,
    date_internal,
    content_file_path,
    has_attachments,
    attachment_count
)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(email.id)
        .bind(email.account_id)
        .bind(&email.provider_id)
        .bind(&email.message_id)
        .bind(&email.subject)
        .bind(email.from.as_ref().map(|from| from.address.clone()))
        .bind(email.from.as_ref().and_then(|from| from.name.clone()))
        .bind(serialize_addresses(&email.to)?)
        .bind(email.date_internal)
        .bind(&email.content_file_path)
        .bind(email.has_attachments)
        .bind(email.attachment_count)
        .execute(self.pool())
        .await?;

        self.get_email(email.id).await
    }

    async fn get_email(&self, email_id: Uuid) -> Result<EmailRecord> {
        let row = sqlx::query(
            r#"
SELECT
    id,
    account_id,
    provider_id,
    message_id,
    subject,
    from_address,
    from_name,
    to_addresses,
    date_internal,
    content_file_path,
    has_attachments,
    attachment_count
FROM emails
WHERE id = ?
            "#,
        )
        .bind(email_id)
        .fetch_optional(self.pool())
        .await?;

        match row {
            Some(row) => email_from_row(row),
            None => Err(Error::NotFound(format!("email {email_id}"))),
        }
    }

    async fn list_emails_for_account(
        &self,
        account_id: Uuid,
        limit: usize,
    ) -> Result<Vec<EmailRecord>> {
        let rows = sqlx::query(
            r#"
SELECT
    id,
    account_id,
    provider_id,
    message_id,
    subject,
    from_address,
    from_name,
    to_addresses,
    date_internal,
    content_file_path,
    has_attachments,
    attachment_count
FROM emails
WHERE account_id = ?
ORDER BY date_internal DESC
LIMIT ?
            "#,
        )
        .bind(account_id)
        .bind(limit as i64)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(email_from_row).collect()
    }

    async fn list_emails_in_folder(
        &self,
        folder_id: Uuid,
        limit: usize,
    ) -> Result<Vec<EmailRecord>> {
        let rows = sqlx::query(
            r#"
SELECT
    e.id,
    e.account_id,
    e.provider_id,
    e.message_id,
    e.subject,
    e.from_address,
    e.from_name,
    e.to_addresses,
    e.date_internal,
    e.content_file_path,
    e.has_attachments,
    e.attachment_count
FROM emails e
INNER JOIN email_folders ef ON ef.email_id = e.id
WHERE ef.unified_folder_id = ?
ORDER BY e.date_internal DESC
LIMIT ?
            "#,
        )
        .bind(folder_id)
        .bind(limit as i64)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(email_from_row).collect()
    }
}

#[async_trait]
impl AttachmentStore for Storage {
    async fn insert_attachment(&self, attachment: NewAttachment) -> Result<AttachmentRecord> {
        sqlx::query(
            r#"
INSERT INTO attachments (id, email_id, filename, mime_type, size_bytes, file_path, file_hash)
VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(attachment.id)
        .bind(attachment.email_id)
        .bind(&attachment.filename)
        .bind(&attachment.mime_type)
        .bind(attachment.size_bytes)
        .bind(&attachment.file_path)
        .bind(&attachment.file_hash)
        .execute(self.pool())
        .await?;

        let rows = self.list_attachments_for_email(attachment.email_id).await?;
        rows.into_iter()
            .find(|record| record.id == attachment.id)
            .ok_or_else(|| Error::NotFound(format!("attachment {}", attachment.id)))
    }

    async fn list_attachments_for_email(&self, email_id: Uuid) -> Result<Vec<AttachmentRecord>> {
        let rows = sqlx::query(
            r#"
SELECT id, email_id, filename, mime_type, size_bytes, file_path, file_hash
FROM attachments
WHERE email_id = ?
ORDER BY filename ASC
            "#,
        )
        .bind(email_id)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(attachment_from_row).collect()
    }
}

fn account_from_row(row: sqlx::sqlite::SqliteRow) -> Result<AccountRecord> {
    Ok(AccountRecord {
        id: row.get("id"),
        email: row.get("email"),
        display_name: row.get("display_name"),
        provider_type: ProviderType::parse(row.get::<&str, _>("provider_type"))?,
    })
}

fn folder_from_row(row: sqlx::sqlite::SqliteRow) -> Result<UnifiedFolderRecord> {
    Ok(UnifiedFolderRecord {
        id: row.get("id"),
        name: row.get("name"),
        folder_type: UnifiedFolderType::parse(row.get::<&str, _>("folder_type"))?,
        parent_id: row.get("parent_id"),
    })
}

fn folder_mapping_from_row(row: sqlx::sqlite::SqliteRow) -> Result<FolderMappingRecord> {
    Ok(FolderMappingRecord {
        id: row.get("id"),
        unified_folder_id: row.get("unified_folder_id"),
        account_id: row.get("account_id"),
        provider_folder_id: row.get("provider_folder_id"),
        provider_folder_name: row.get("provider_folder_name"),
    })
}

fn email_from_row(row: sqlx::sqlite::SqliteRow) -> Result<EmailRecord> {
    let from_address: Option<String> = row.get("from_address");
    let from_name: Option<String> = row.get("from_name");
    let serialized_to: String = row.get("to_addresses");

    Ok(EmailRecord {
        id: row.get("id"),
        account_id: row.get("account_id"),
        provider_id: row.get("provider_id"),
        message_id: row.get("message_id"),
        subject: row.get("subject"),
        from: from_address.map(|address| EmailAddress {
            address,
            name: from_name,
        }),
        to: deserialize_addresses(&serialized_to)?,
        date_internal: row.get::<DateTime<Utc>, _>("date_internal"),
        content_file_path: row.get("content_file_path"),
        has_attachments: row.get("has_attachments"),
        attachment_count: row.get("attachment_count"),
    })
}

fn attachment_from_row(row: sqlx::sqlite::SqliteRow) -> Result<AttachmentRecord> {
    Ok(AttachmentRecord {
        id: row.get("id"),
        email_id: row.get("email_id"),
        filename: row.get("filename"),
        mime_type: row.get("mime_type"),
        size_bytes: row.get("size_bytes"),
        file_path: row.get("file_path"),
        file_hash: row.get("file_hash"),
    })
}

fn serialize_addresses(addresses: &[EmailAddress]) -> Result<String> {
    serde_json::to_string(addresses).map_err(Error::from)
}

fn deserialize_addresses(serialized: &str) -> Result<Vec<EmailAddress>> {
    serde_json::from_str(serialized).map_err(Error::from)
}
