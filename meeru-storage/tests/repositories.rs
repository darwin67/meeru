use chrono::{Duration, Utc};
use meeru_storage::{
    AccountStore, AttachmentStore, EmailAddress, EmailStore, FolderStore, NewAccount,
    NewAttachment, NewEmail, NewFolderMapping, NewUnifiedFolder, ProviderType, StorageConfig,
    UnifiedFolderType,
};
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn account_store_supports_create_list_get_delete() {
    let temp_dir = TempDir::new().expect("temp dir");
    let storage = StorageConfig::new(temp_dir.path())
        .open()
        .await
        .expect("open storage");

    let account = NewAccount {
        id: Uuid::new_v4(),
        email: "alice@example.com".to_string(),
        display_name: Some("Alice".to_string()),
        provider_type: ProviderType::Generic,
    };

    let created = storage
        .create_account(account.clone())
        .await
        .expect("create account");
    let listed = storage.list_accounts().await.expect("list accounts");
    let fetched = storage.get_account(account.id).await.expect("get account");

    assert_eq!(created, fetched);
    assert_eq!(listed, vec![created.clone()]);

    storage
        .delete_account(account.id)
        .await
        .expect("delete account");
    assert!(storage
        .list_accounts()
        .await
        .expect("list accounts")
        .is_empty());
}

#[tokio::test]
async fn folder_email_and_attachment_queries_round_trip() {
    let temp_dir = TempDir::new().expect("temp dir");
    let storage = StorageConfig::new(temp_dir.path())
        .open()
        .await
        .expect("open storage");

    let account = storage
        .create_account(NewAccount {
            id: Uuid::new_v4(),
            email: "bob@example.com".to_string(),
            display_name: Some("Bob".to_string()),
            provider_type: ProviderType::Generic,
        })
        .await
        .expect("create account");

    let inbox = storage
        .create_unified_folder(NewUnifiedFolder {
            id: Uuid::new_v4(),
            name: "Inbox".to_string(),
            folder_type: UnifiedFolderType::Inbox,
            parent_id: None,
        })
        .await
        .expect("create folder");

    let mapping = storage
        .create_folder_mapping(NewFolderMapping {
            id: Uuid::new_v4(),
            unified_folder_id: inbox.id,
            account_id: account.id,
            provider_folder_id: "INBOX".to_string(),
            provider_folder_name: Some("Inbox".to_string()),
        })
        .await
        .expect("create folder mapping");

    let older_email = storage
        .insert_email(NewEmail {
            id: Uuid::new_v4(),
            account_id: account.id,
            provider_id: "42".to_string(),
            message_id: Some("<42@example.com>".to_string()),
            subject: Some("Older".to_string()),
            from: Some(EmailAddress {
                address: "sender@example.com".to_string(),
                name: Some("Sender".to_string()),
            }),
            to: vec![EmailAddress {
                address: "bob@example.com".to_string(),
                name: Some("Bob".to_string()),
            }],
            date_internal: Utc::now() - Duration::hours(1),
            content_file_path: Some("blobs/emails/older.eml".to_string()),
            has_attachments: false,
            attachment_count: 0,
        })
        .await
        .expect("insert older email");

    let newer_email = storage
        .insert_email(NewEmail {
            id: Uuid::new_v4(),
            account_id: account.id,
            provider_id: "43".to_string(),
            message_id: Some("<43@example.com>".to_string()),
            subject: Some("Newer".to_string()),
            from: Some(EmailAddress {
                address: "sender@example.com".to_string(),
                name: Some("Sender".to_string()),
            }),
            to: vec![EmailAddress {
                address: "bob@example.com".to_string(),
                name: Some("Bob".to_string()),
            }],
            date_internal: Utc::now(),
            content_file_path: Some("blobs/emails/newer.eml".to_string()),
            has_attachments: true,
            attachment_count: 1,
        })
        .await
        .expect("insert newer email");

    storage
        .assign_email_to_folder(older_email.id, inbox.id)
        .await
        .expect("assign older email");
    storage
        .assign_email_to_folder(newer_email.id, inbox.id)
        .await
        .expect("assign newer email");

    let attachment = storage
        .insert_attachment(NewAttachment {
            id: Uuid::new_v4(),
            email_id: newer_email.id,
            filename: "invoice.pdf".to_string(),
            mime_type: Some("application/pdf".to_string()),
            size_bytes: Some(1_024),
            file_path: Some("blobs/attachments/invoice.pdf".to_string()),
            file_hash: Some("hash-123".to_string()),
        })
        .await
        .expect("insert attachment");

    let account_emails = storage
        .list_emails_for_account(account.id, 10)
        .await
        .expect("list account emails");
    let folder_emails = storage
        .list_emails_in_folder(inbox.id, 10)
        .await
        .expect("list folder emails");
    let attachments = storage
        .list_attachments_for_email(newer_email.id)
        .await
        .expect("list attachments");
    let mappings = storage
        .list_folder_mappings_for_account(account.id)
        .await
        .expect("list mappings");

    assert_eq!(mappings, vec![mapping]);
    assert_eq!(attachments, vec![attachment]);
    assert_eq!(account_emails.len(), 2);
    assert_eq!(folder_emails.len(), 2);
    assert_eq!(account_emails[0].id, newer_email.id);
    assert_eq!(account_emails[1].id, older_email.id);
    assert_eq!(folder_emails[0].id, newer_email.id);
    assert_eq!(folder_emails[1].id, older_email.id);
}
