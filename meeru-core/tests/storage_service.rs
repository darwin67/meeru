use meeru_core::{
    account::{Account, ProviderType},
    email::{Email, EmailAddress},
    storage::{StorageService, SyncedAttachment, SyncedEmail},
    unified::{UnifiedFolder, UnifiedFolderType},
};
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn storage_service_supports_generic_account_setup() {
    let temp_dir = TempDir::new().expect("temp dir");
    let service = StorageService::open(meeru_storage::StorageConfig::new(temp_dir.path()))
        .await
        .expect("open service");

    let created = service
        .add_account(Account {
            id: Uuid::new_v4(),
            email: "alice@example.com".to_string(),
            display_name: Some("Alice".to_string()),
            provider_type: ProviderType::Generic,
        })
        .await
        .expect("create account");

    let updated = service
        .update_account(Account {
            display_name: Some("Alice Updated".to_string()),
            ..created.clone()
        })
        .await
        .expect("update account");

    let accounts = service.list_accounts().await.expect("list accounts");

    assert_eq!(accounts, vec![updated]);
}

#[tokio::test]
async fn storage_service_supports_first_pass_sync_caching() {
    let temp_dir = TempDir::new().expect("temp dir");
    let service = StorageService::open(meeru_storage::StorageConfig::new(temp_dir.path()))
        .await
        .expect("open service");

    let account = service
        .add_account(Account {
            id: Uuid::new_v4(),
            email: "sync@example.com".to_string(),
            display_name: Some("Sync".to_string()),
            provider_type: ProviderType::Generic,
        })
        .await
        .expect("create account");

    let inbox = service
        .create_unified_folder(UnifiedFolder {
            id: Uuid::new_v4(),
            name: "Inbox".to_string(),
            folder_type: UnifiedFolderType::Inbox,
            parent_id: None,
        })
        .await
        .expect("create folder");

    let email_id = Uuid::new_v4();
    let cached = service
        .cache_synced_email(SyncedEmail {
            email: Email {
                id: email_id,
                account_id: account.id,
                provider_id: "provider-100".to_string(),
                message_id: Some("<provider-100@example.com>".to_string()),
                subject: Some("Welcome".to_string()),
                from: Some(EmailAddress {
                    address: "sender@example.com".to_string(),
                    name: Some("Sender".to_string()),
                }),
                to: vec![EmailAddress {
                    address: "sync@example.com".to_string(),
                    name: Some("Sync".to_string()),
                }],
                date: chrono::Utc::now(),
                content_ref: None,
                content: None,
                has_attachments: true,
                attachment_count: 1,
            },
            body: b"hello from sync".to_vec(),
            folder_ids: vec![inbox.id],
            attachments: vec![SyncedAttachment {
                id: Uuid::new_v4(),
                filename: "welcome.txt".to_string(),
                mime_type: Some("text/plain".to_string()),
                content: b"attachment".to_vec(),
            }],
        })
        .await
        .expect("cache synced email");

    let listed = service
        .list_folder_emails(inbox.id, 10)
        .await
        .expect("list folder emails");
    let body = service
        .load_email_body(&cached)
        .await
        .expect("load email body");
    let attachments = service
        .list_attachments_for_email(email_id)
        .await
        .expect("list attachments");

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, email_id);
    assert_eq!(body.text.as_deref(), Some("hello from sync"));
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].filename, "welcome.txt");
}
