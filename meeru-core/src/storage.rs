//! Storage-backed core services and model conversions.

use uuid::Uuid;

use crate::{
    account::{Account, ProviderType},
    email::{Email, EmailAddress, EmailContent},
    unified::{UnifiedFolder, UnifiedFolderType},
    Result,
};
use meeru_storage::{
    AccountStore, AttachmentStore, BlobStore, EmailStore, FolderStore, NewAttachment, NewEmail,
    NewEmailGraph, NewUnifiedFolder, Storage, StorageConfig,
};

#[derive(Debug, Clone)]
pub struct StorageService {
    storage: Storage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncedAttachment {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: Option<String>,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyncedEmail {
    pub email: Email,
    pub body: Vec<u8>,
    pub folder_ids: Vec<Uuid>,
    pub attachments: Vec<SyncedAttachment>,
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

    pub async fn cache_synced_email(&self, synced: SyncedEmail) -> Result<Email> {
        let body_path = self
            .storage
            .put_email_body(synced.email.id, &synced.body)
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

        let graph = NewEmailGraph {
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

        match self.storage.insert_email_graph(graph).await {
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

    pub async fn load_email_body(&self, email: &Email) -> Result<EmailContent> {
        let content_ref = email
            .content_ref
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("email {} has no content reference", email.id))?;
        let body = self.storage.read_blob(content_ref).await?;
        let text = String::from_utf8(body).map_err(anyhow::Error::from)?;

        Ok(EmailContent {
            text: Some(text),
            html: None,
        })
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
