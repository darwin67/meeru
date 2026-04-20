//! Storage-backed core services and model conversions.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    account::{Account, ProviderType},
    email::{Email, EmailAddress, EmailContent},
    unified::{UnifiedFolder, UnifiedFolderType},
    Result,
};
use meeru_providers::{parse_raw_message, FetchedMessage, ParsedMessage};
use meeru_storage::{
    AccountStore, AttachmentStore, BlobStore, EmailStore, FolderMappingRecord, FolderStore,
    NewAttachment, NewEmail, NewEmailBundle, NewFolderMapping, NewUnifiedFolder, Storage,
    StorageConfig,
};

/// Core façade over the storage layer used by account, folder, and sync workflows.
#[derive(Debug, Clone)]
pub struct StorageService {
    /// Open storage handle shared across repository operations.
    storage: Storage,
}

/// Attachment materialized from synced storage for message reads and tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncedAttachment {
    /// Stable local identifier for the stored attachment blob.
    pub id: Uuid,
    /// Attachment filename shown in the UI and used for blob naming.
    pub filename: String,
    /// MIME type recorded for the attachment when known.
    pub mime_type: Option<String>,
    /// Attachment payload bytes loaded from storage or parser output.
    pub content: Vec<u8>,
}

/// Parsed message plus storage metadata ready to be written into the local bundle insert.
#[derive(Debug, Clone, PartialEq)]
pub struct SyncedEmail {
    /// Canonical email record that will be inserted into the email table.
    pub email: Email,
    /// Raw RFC822 bytes stored as the canonical body blob.
    pub raw_message: Vec<u8>,
    /// Unified folder ids that should reference the email after caching.
    pub folder_ids: Vec<Uuid>,
    /// Attachments to persist alongside the email bundle insert.
    pub attachments: Vec<SyncedAttachment>,
}

impl SyncedEmail {
    /// Build a storage-ready synced email from parsed raw message content and sync metadata.
    pub fn from_parsed_message(
        email_id: Uuid,
        account_id: Uuid,
        provider_id: String,
        folder_ids: Vec<Uuid>,
        raw_message: Vec<u8>,
        parsed: ParsedMessage,
        fallback_date: DateTime<Utc>,
    ) -> Self {
        let attachments: Vec<_> = parsed
            .attachments
            .into_iter()
            .map(|attachment| SyncedAttachment {
                id: Uuid::new_v4(),
                filename: attachment.filename,
                mime_type: attachment.mime_type,
                content: attachment.content,
            })
            .collect();

        let attachment_count = attachments.len() as i64;

        Self {
            email: Email {
                id: email_id,
                account_id,
                provider_id,
                message_id: parsed.message_id,
                subject: parsed.subject,
                from: parsed.from.map(|from| EmailAddress {
                    address: from.address,
                    name: from.name,
                }),
                to: parsed
                    .to
                    .into_iter()
                    .map(|to| EmailAddress {
                        address: to.address,
                        name: to.name,
                    })
                    .collect(),
                date: parsed.date.unwrap_or(fallback_date),
                content_ref: None,
                content: None,
                has_attachments: attachment_count > 0,
                attachment_count,
            },
            raw_message,
            folder_ids,
            attachments,
        }
    }
}

impl StorageService {
    pub async fn open(config: StorageConfig) -> Result<Self> {
        let storage = config.open().await?;
        Ok(Self { storage })
    }

    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub async fn add_account(&self, account: Account) -> Result<Account> {
        let created = self
            .storage
            .create_account(account_to_storage_new(account))
            .await?;
        Ok(created.into())
    }

    pub async fn update_account(&self, account: Account) -> Result<Account> {
        let updated = self
            .storage
            .update_account(account_to_storage_record(account))
            .await?;
        Ok(updated.into())
    }

    pub async fn list_accounts(&self) -> Result<Vec<Account>> {
        let accounts = self.storage.list_accounts().await?;
        Ok(accounts.into_iter().map(Into::into).collect())
    }

    pub async fn create_unified_folder(&self, folder: UnifiedFolder) -> Result<UnifiedFolder> {
        let created = self
            .storage
            .create_unified_folder(folder_to_storage(folder))
            .await?;
        Ok(created.into())
    }

    pub async fn create_folder_mapping(
        &self,
        account_id: Uuid,
        unified_folder_id: Uuid,
        provider_folder_id: String,
        provider_folder_name: Option<String>,
    ) -> Result<FolderMappingRecord> {
        Ok(self
            .storage
            .create_folder_mapping(NewFolderMapping {
                id: Uuid::new_v4(),
                unified_folder_id,
                account_id,
                provider_folder_id,
                provider_folder_name,
            })
            .await?)
    }

    pub async fn list_folder_mappings_for_account(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<FolderMappingRecord>> {
        Ok(self
            .storage
            .list_folder_mappings_for_account(account_id)
            .await?)
    }

    pub async fn cache_synced_email(&self, synced: SyncedEmail) -> Result<Email> {
        let body_path = self
            .storage
            .put_email_body(synced.email.id, &synced.raw_message)
            .await?;

        let mut attachment_paths = Vec::new();
        for attachment in &synced.attachments {
            let path = self
                .storage
                .put_attachment_payload(attachment.id, &attachment.filename, &attachment.content)
                .await?;
            attachment_paths.push((attachment.clone(), path));
        }

        let mut email = synced.email.clone();
        email.content_ref = Some(body_path.clone());
        email.has_attachments = !attachment_paths.is_empty();
        email.attachment_count = attachment_paths.len() as i64;

        let bundle = NewEmailBundle {
            email: email_to_storage_new(email),
            folder_ids: synced.folder_ids,
            attachments: attachment_paths
                .iter()
                .map(|(attachment, path)| NewAttachment {
                    id: attachment.id,
                    email_id: synced.email.id,
                    filename: attachment.filename.clone(),
                    mime_type: attachment.mime_type.clone(),
                    size_bytes: Some(attachment.content.len() as i64),
                    file_path: Some(path.clone()),
                    file_hash: None,
                })
                .collect(),
        };

        match self.storage.insert_email_bundle(bundle).await {
            Ok(record) => Ok(record.into()),
            Err(error) => {
                let _ = self.storage.delete_blob(&body_path).await;
                for (_, path) in attachment_paths {
                    let _ = self.storage.delete_blob(&path).await;
                }
                Err(error.into())
            },
        }
    }

    pub async fn list_folder_emails(&self, folder_id: Uuid, limit: usize) -> Result<Vec<Email>> {
        let emails = self.storage.list_emails_in_folder(folder_id, limit).await?;
        Ok(emails.into_iter().map(Into::into).collect())
    }

    pub async fn sync_fetched_messages(
        &self,
        account_id: Uuid,
        folder_id: Uuid,
        fetched_messages: Vec<FetchedMessage>,
    ) -> Result<Vec<Email>> {
        let mut synced_emails = Vec::with_capacity(fetched_messages.len());

        for fetched in fetched_messages {
            let provider_id = fetched.identity.provider_id();

            match self
                .storage
                .get_email_by_provider_id(account_id, &provider_id)
                .await
            {
                Ok(existing) => {
                    self.storage
                        .assign_email_to_folder(existing.id, folder_id)
                        .await?;
                    synced_emails.push(existing.into());
                },
                Err(meeru_storage::Error::NotFound(_)) => {
                    let parsed = parse_raw_message(&fetched.raw_message)?;
                    let synced = SyncedEmail::from_parsed_message(
                        Uuid::new_v4(),
                        account_id,
                        provider_id,
                        vec![folder_id],
                        fetched.raw_message,
                        parsed,
                        Utc::now(),
                    );
                    synced_emails.push(self.cache_synced_email(synced).await?);
                },
                Err(error) => return Err(error.into()),
            }
        }

        Ok(synced_emails)
    }

    pub async fn sync_provider_mailbox(
        &self,
        account_id: Uuid,
        provider_folder_id: &str,
        fetched_messages: Vec<FetchedMessage>,
    ) -> Result<Vec<Email>> {
        let mappings = self
            .storage
            .list_folder_mappings_for_account(account_id)
            .await?;
        let mapping = mappings
            .into_iter()
            .find(|mapping| mapping.provider_folder_id == provider_folder_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "no folder mapping found for account {} and provider folder {}",
                    account_id,
                    provider_folder_id
                )
            })?;

        self.sync_fetched_messages(account_id, mapping.unified_folder_id, fetched_messages)
            .await
    }

    pub async fn load_email_body(&self, email: &Email) -> Result<EmailContent> {
        let content_ref = email
            .content_ref
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("email {} has no content reference", email.id))?;
        let raw_message = self.storage.read_blob(content_ref).await?;

        match parse_raw_message(&raw_message) {
            Ok(parsed) => Ok(EmailContent {
                text: parsed.text_body,
                html: parsed.html_body,
            }),
            Err(_) => {
                let text = String::from_utf8(raw_message).map_err(anyhow::Error::from)?;
                Ok(EmailContent {
                    text: Some(text),
                    html: None,
                })
            },
        }
    }

    pub async fn list_attachments_for_email(
        &self,
        email_id: Uuid,
    ) -> Result<Vec<meeru_storage::AttachmentRecord>> {
        Ok(self.storage.list_attachments_for_email(email_id).await?)
    }
}

impl From<meeru_storage::AccountRecord> for Account {
    fn from(value: meeru_storage::AccountRecord) -> Self {
        Self {
            id: value.id,
            email: value.email,
            display_name: value.display_name,
            provider_type: match value.provider_type {
                meeru_storage::ProviderType::Gmail => ProviderType::Gmail,
                meeru_storage::ProviderType::Outlook => ProviderType::Outlook,
                meeru_storage::ProviderType::Generic => ProviderType::Generic,
            },
        }
    }
}

impl From<meeru_storage::UnifiedFolderRecord> for UnifiedFolder {
    fn from(value: meeru_storage::UnifiedFolderRecord) -> Self {
        Self {
            id: value.id,
            name: value.name,
            folder_type: match value.folder_type {
                meeru_storage::UnifiedFolderType::Inbox => UnifiedFolderType::Inbox,
                meeru_storage::UnifiedFolderType::Sent => UnifiedFolderType::Sent,
                meeru_storage::UnifiedFolderType::Drafts => UnifiedFolderType::Drafts,
                meeru_storage::UnifiedFolderType::Archive => UnifiedFolderType::Archive,
                meeru_storage::UnifiedFolderType::Trash => UnifiedFolderType::Trash,
                meeru_storage::UnifiedFolderType::Spam => UnifiedFolderType::Spam,
                meeru_storage::UnifiedFolderType::Custom => UnifiedFolderType::Custom,
            },
            parent_id: value.parent_id,
        }
    }
}

impl From<meeru_storage::EmailRecord> for Email {
    fn from(value: meeru_storage::EmailRecord) -> Self {
        Self {
            id: value.id,
            account_id: value.account_id,
            provider_id: value.provider_id,
            message_id: value.message_id,
            subject: value.subject,
            from: value.from.map(Into::into),
            to: value.to.into_iter().map(Into::into).collect(),
            date: value.date_internal,
            content_ref: value.content_file_path,
            content: None,
            has_attachments: value.has_attachments,
            attachment_count: value.attachment_count,
        }
    }
}

impl From<meeru_storage::EmailAddress> for EmailAddress {
    fn from(value: meeru_storage::EmailAddress) -> Self {
        Self {
            address: value.address,
            name: value.name,
        }
    }
}

fn account_to_storage_new(account: Account) -> meeru_storage::NewAccount {
    meeru_storage::NewAccount {
        id: account.id,
        email: account.email,
        display_name: account.display_name,
        provider_type: provider_type_to_storage(account.provider_type),
    }
}

fn account_to_storage_record(account: Account) -> meeru_storage::AccountRecord {
    meeru_storage::AccountRecord {
        id: account.id,
        email: account.email,
        display_name: account.display_name,
        provider_type: provider_type_to_storage(account.provider_type),
    }
}

fn folder_to_storage(folder: UnifiedFolder) -> NewUnifiedFolder {
    NewUnifiedFolder {
        id: folder.id,
        name: folder.name,
        folder_type: match folder.folder_type {
            UnifiedFolderType::Inbox => meeru_storage::UnifiedFolderType::Inbox,
            UnifiedFolderType::Sent => meeru_storage::UnifiedFolderType::Sent,
            UnifiedFolderType::Drafts => meeru_storage::UnifiedFolderType::Drafts,
            UnifiedFolderType::Archive => meeru_storage::UnifiedFolderType::Archive,
            UnifiedFolderType::Trash => meeru_storage::UnifiedFolderType::Trash,
            UnifiedFolderType::Spam => meeru_storage::UnifiedFolderType::Spam,
            UnifiedFolderType::Custom => meeru_storage::UnifiedFolderType::Custom,
        },
        parent_id: folder.parent_id,
    }
}

fn email_to_storage_new(email: Email) -> NewEmail {
    NewEmail {
        id: email.id,
        account_id: email.account_id,
        provider_id: email.provider_id,
        message_id: email.message_id,
        subject: email.subject,
        from: email.from.map(email_address_to_storage),
        to: email.to.into_iter().map(email_address_to_storage).collect(),
        date_internal: email.date,
        content_file_path: email.content_ref,
        has_attachments: email.has_attachments,
        attachment_count: email.attachment_count,
    }
}

fn email_address_to_storage(address: EmailAddress) -> meeru_storage::EmailAddress {
    meeru_storage::EmailAddress {
        address: address.address,
        name: address.name,
    }
}

fn provider_type_to_storage(provider_type: ProviderType) -> meeru_storage::ProviderType {
    match provider_type {
        ProviderType::Gmail => meeru_storage::ProviderType::Gmail,
        ProviderType::Outlook => meeru_storage::ProviderType::Outlook,
        ProviderType::Generic => meeru_storage::ProviderType::Generic,
    }
}
