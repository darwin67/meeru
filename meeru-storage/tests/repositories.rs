use chrono::{Duration, Utc};
use meeru_storage::{
    AccountStore, AttachmentStore, EmailAddress, EmailStore, FolderStore, NewAccount,
    NewAttachment, NewEmail, NewEmailBundle, NewFolderMapping, NewUnifiedFolder, ProviderType,
    StorageConfig, UnifiedFolderType,
};
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn account_store_supports_create_list_get_update_delete() {
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
    let updated = storage
        .update_account(meeru_storage::AccountRecord {
            display_name: Some("Alice Updated".to_string()),
            ..created.clone()
        })
        .await
        .expect("update account");
    let listed = storage.list_accounts().await.expect("list accounts");
    let fetched = storage.get_account(account.id).await.expect("get account");

    assert_eq!(updated, fetched);
    assert_eq!(listed, vec![updated.clone()]);
    assert_eq!(updated.display_name.as_deref(), Some("Alice Updated"));

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
    let fetched_by_provider_id = storage
        .get_email_by_provider_id(account.id, "43")
        .await
        .expect("get email by provider id");
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
    assert_eq!(fetched_by_provider_id.id, newer_email.id);
    assert_eq!(account_emails.len(), 2);
    assert_eq!(folder_emails.len(), 2);
    assert_eq!(account_emails[0].id, newer_email.id);
    assert_eq!(account_emails[1].id, older_email.id);
    assert_eq!(folder_emails[0].id, newer_email.id);
    assert_eq!(folder_emails[1].id, older_email.id);
}

#[tokio::test]
async fn email_store_supports_update_and_transactional_bundle_writes() {
    let temp_dir = TempDir::new().expect("temp dir");
    let storage = StorageConfig::new(temp_dir.path())
        .open()
        .await
        .expect("open storage");

    let account = storage
        .create_account(NewAccount {
            id: Uuid::new_v4(),
            email: "sync@example.com".to_string(),
            display_name: Some("Sync".to_string()),
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

    let email_id = Uuid::new_v4();
    let inserted = storage
        .insert_email_bundle(NewEmailBundle {
            email: NewEmail {
                id: email_id,
                account_id: account.id,
                provider_id: "55".to_string(),
                message_id: Some("<55@example.com>".to_string()),
                subject: Some("Original".to_string()),
                from: Some(EmailAddress {
                    address: "sender@example.com".to_string(),
                    name: Some("Sender".to_string()),
                }),
                to: vec![EmailAddress {
                    address: "sync@example.com".to_string(),
                    name: Some("Sync".to_string()),
                }],
                date_internal: Utc::now(),
                content_file_path: Some("blobs/emails/original.eml".to_string()),
                has_attachments: true,
                attachment_count: 1,
            },
            folder_ids: vec![inbox.id],
            attachments: vec![NewAttachment {
                id: Uuid::new_v4(),
                email_id,
                filename: "sync.pdf".to_string(),
                mime_type: Some("application/pdf".to_string()),
                size_bytes: Some(512),
                file_path: Some("blobs/attachments/sync.pdf".to_string()),
                file_hash: Some("hash-sync".to_string()),
            }],
        })
        .await
        .expect("insert email bundle");

    let updated = storage
        .update_email(meeru_storage::EmailRecord {
            subject: Some("Updated".to_string()),
            attachment_count: 2,
            ..inserted.clone()
        })
        .await
        .expect("update email");

    let fetched = storage.get_email(email_id).await.expect("fetch email");
    let attachments = storage
        .list_attachments_for_email(email_id)
        .await
        .expect("list attachments");
    let folder_emails = storage
        .list_emails_in_folder(inbox.id, 10)
        .await
        .expect("list folder emails");

    assert_eq!(updated, fetched);
    assert_eq!(updated.subject.as_deref(), Some("Updated"));
    assert_eq!(updated.attachment_count, 2);
    assert_eq!(attachments.len(), 1);
    assert_eq!(folder_emails.len(), 1);
    assert_eq!(folder_emails[0].id, email_id);
}

#[tokio::test]
async fn transactional_bundle_writes_roll_back_on_folder_errors() {
    let temp_dir = TempDir::new().expect("temp dir");
    let storage = StorageConfig::new(temp_dir.path())
        .open()
        .await
        .expect("open storage");

    let account = storage
        .create_account(NewAccount {
            id: Uuid::new_v4(),
            email: "rollback@example.com".to_string(),
            display_name: None,
            provider_type: ProviderType::Generic,
        })
        .await
        .expect("create account");

    let email_id = Uuid::new_v4();
    let error = storage
        .insert_email_bundle(NewEmailBundle {
            email: NewEmail {
                id: email_id,
                account_id: account.id,
                provider_id: "rollback-1".to_string(),
                message_id: None,
                subject: Some("Rollback".to_string()),
                from: None,
                to: vec![],
                date_internal: Utc::now(),
                content_file_path: None,
                has_attachments: true,
                attachment_count: 1,
            },
            folder_ids: vec![Uuid::new_v4()],
            attachments: vec![NewAttachment {
                id: Uuid::new_v4(),
                email_id,
                filename: "rollback.txt".to_string(),
                mime_type: Some("text/plain".to_string()),
                size_bytes: Some(12),
                file_path: Some("blobs/attachments/rollback.txt".to_string()),
                file_hash: None,
            }],
        })
        .await
        .expect_err("insert email bundle should fail");

    assert!(matches!(error, meeru_storage::Error::Database(_)));
    assert!(storage
        .list_emails_for_account(account.id, 10)
        .await
        .expect("list emails")
        .is_empty());
}
