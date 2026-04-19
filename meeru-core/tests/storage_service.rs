use meeru_core::{
    account::{Account, ProviderType},
    email::{Email, EmailAddress},
    storage::{StorageService, SyncedAttachment, SyncedEmail},
    unified::{UnifiedFolder, UnifiedFolderType},
};
use meeru_providers::{parse_rfc822_message, FetchedMessage, ImapMessageIdentity};
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
            raw_message: concat!(
                "From: Sender <sender@example.com>\r\n",
                "To: Sync <sync@example.com>\r\n",
                "Subject: Welcome\r\n",
                "Message-ID: <provider-100@example.com>\r\n",
                "Date: Sat, 19 Apr 2026 10:30:00 +0000\r\n",
                "MIME-Version: 1.0\r\n",
                "Content-Type: multipart/alternative; boundary=\"alt\"\r\n",
                "\r\n",
                "--alt\r\n",
                "Content-Type: text/plain; charset=\"utf-8\"\r\n",
                "\r\n",
                "hello from sync\r\n",
                "--alt\r\n",
                "Content-Type: text/html; charset=\"utf-8\"\r\n",
                "\r\n",
                "<html><body><p>hello from sync</p></body></html>\r\n",
                "--alt--\r\n"
            )
            .as_bytes()
            .to_vec(),
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
    assert_eq!(
        body.html.as_deref(),
        Some("<html><body><p>hello from sync</p></body></html>")
    );
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].filename, "welcome.txt");
}

#[test]
fn synced_email_can_be_built_from_parsed_provider_data() {
    let raw_message = concat!(
        "From: Sender <sender@example.com>\r\n",
        "To: Recipient <recipient@example.com>\r\n",
        "Subject: Parsed conversion\r\n",
        "Message-ID: <parsed@example.com>\r\n",
        "Date: Sat, 19 Apr 2026 10:30:00 +0000\r\n",
        "Content-Type: text/plain; charset=\"utf-8\"\r\n",
        "\r\n",
        "hello from parsed sync\r\n"
    )
    .as_bytes()
    .to_vec();
    let parsed = parse_rfc822_message(&raw_message).expect("message should parse");

    let synced = SyncedEmail::from_parsed_message(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "INBOX:42:7".to_string(),
        vec![Uuid::new_v4()],
        raw_message,
        parsed,
        chrono::Utc::now(),
    );

    assert_eq!(synced.email.subject.as_deref(), Some("Parsed conversion"));
    assert_eq!(synced.email.message_id.as_deref(), Some("parsed@example.com"));
    assert_eq!(synced.email.to.len(), 1);
    assert_eq!(synced.email.to[0].address, "recipient@example.com");
    assert_eq!(synced.attachments.len(), 0);
}

#[tokio::test]
async fn storage_service_syncs_fetched_messages_idempotently() {
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
    service
        .create_folder_mapping(
            account.id,
            inbox.id,
            "INBOX".to_string(),
            Some("Inbox".to_string()),
        )
        .await
        .expect("create folder mapping");

    let raw_message = concat!(
        "From: Sender <sender@example.com>\r\n",
        "To: Sync <sync@example.com>\r\n",
        "Subject: Synced once\r\n",
        "Message-ID: <sync-once@example.com>\r\n",
        "Date: Sat, 19 Apr 2026 10:30:00 +0000\r\n",
        "Content-Type: text/plain; charset=\"utf-8\"\r\n",
        "\r\n",
        "idempotent sync body\r\n"
    )
    .as_bytes()
    .to_vec();

    let fetched = FetchedMessage {
        identity: ImapMessageIdentity::new("INBOX", 99, 7),
        raw_message,
    };

    let first = service
        .sync_provider_mailbox(account.id, "INBOX", vec![fetched.clone()])
        .await
        .expect("first sync");
    let second = service
        .sync_provider_mailbox(account.id, "INBOX", vec![fetched])
        .await
        .expect("second sync");
    let listed = service
        .list_folder_emails(inbox.id, 10)
        .await
        .expect("list folder emails");

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_eq!(first[0].id, second[0].id);
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].subject.as_deref(), Some("Synced once"));
}

#[tokio::test]
async fn storage_service_requires_folder_mapping_for_provider_mailbox_sync() {
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

    let fetched = FetchedMessage {
        identity: ImapMessageIdentity::new("Archive", 99, 7),
        raw_message: concat!(
            "From: Sender <sender@example.com>\r\n",
            "To: Sync <sync@example.com>\r\n",
            "Subject: Missing mapping\r\n",
            "Date: Sat, 19 Apr 2026 10:30:00 +0000\r\n",
            "Content-Type: text/plain; charset=\"utf-8\"\r\n",
            "\r\n",
            "missing mapping\r\n"
        )
        .as_bytes()
        .to_vec(),
    };

    let error = service
        .sync_provider_mailbox(account.id, "Archive", vec![fetched])
        .await
        .expect_err("sync should require an existing folder mapping");

    assert!(error
        .to_string()
        .contains("no folder mapping found for account"));
}
