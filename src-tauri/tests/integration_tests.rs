use meeru_lib::accounts::AccountManager;
use meeru_lib::db::models::AuthType;
use meeru_lib::db::Database;
use meeru_lib::email::imap_test::ImapTestClient;
use meeru_lib::email::smtp_test::SmtpTestClient;
use meeru_lib::email::{EmailData, EmailSyncService};
use std::time::Duration;
use tempfile::TempDir;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::{runners::AsyncRunner, GenericImage};
use tokio::time::sleep;

/// Greenmail test container configuration
const GREENMAIL_IMAGE: &str = "greenmail/standalone";
const GREENMAIL_TAG: &str = "2.0.1";

// Greenmail ports
const SMTP_PORT: u16 = 3025;
const IMAP_PORT: u16 = 3143;

struct TestEnvironment {
    _temp_dir: TempDir,
    db: Database,
    account_manager: AccountManager,
    sync_service: EmailSyncService,
    _container: testcontainers::ContainerAsync<GenericImage>,
    smtp_port: u16,
    imap_port: u16,
}

impl TestEnvironment {
    async fn new() -> Self {
        // Start Greenmail container
        let container = GenericImage::new(GREENMAIL_IMAGE, GREENMAIL_TAG)
            .with_wait_for(WaitFor::message_on_stdout("Starting GreenMail"))
            .with_exposed_port(SMTP_PORT.tcp())
            .with_exposed_port(IMAP_PORT.tcp())
            .start()
            .await
            .expect("Failed to start container");

        let smtp_port = container
            .get_host_port_ipv4(SMTP_PORT)
            .await
            .expect("Failed to get SMTP port");
        let imap_port = container
            .get_host_port_ipv4(IMAP_PORT)
            .await
            .expect("Failed to get IMAP port");

        // Wait for services to be ready
        sleep(Duration::from_secs(3)).await;

        // Create temporary database
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path).await.unwrap();

        let account_manager = AccountManager::new(db.pool().clone());
        let sync_service = EmailSyncService::new(db.pool().clone());

        Self {
            _temp_dir: temp_dir,
            db,
            account_manager,
            sync_service,
            _container: container,
            smtp_port,
            imap_port,
        }
    }

    fn get_imap_host(&self) -> String {
        "127.0.0.1".to_string()
    }

    fn get_smtp_host(&self) -> String {
        "127.0.0.1".to_string()
    }
}

#[tokio::test]
async fn test_create_account() {
    let env = TestEnvironment::new().await;

    let account = env
        .account_manager
        .create_account(
            "test@localhost".to_string(),
            Some("Test User".to_string()),
            "greenmail".to_string(),
            env.get_imap_host(),
            env.imap_port,
            env.get_smtp_host(),
            env.smtp_port,
            AuthType::Password,
            Some("test".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(account.email, "test@localhost");
    assert_eq!(account.name, Some("Test User".to_string()));

    // Note: In CI/tests, keychain may not work, so we skip password verification

    // Cleanup
    env.account_manager.delete_account(&account.id).await.unwrap();
}

#[tokio::test]
async fn test_list_accounts() {
    let env = TestEnvironment::new().await;

    // Create multiple accounts
    let account1 = env
        .account_manager
        .create_account(
            "user1@localhost".to_string(),
            None,
            "greenmail".to_string(),
            env.get_imap_host(),
            env.imap_port,
            env.get_smtp_host(),
            env.smtp_port,
            AuthType::Password,
            Some("pass1".to_string()),
        )
        .await
        .unwrap();

    let account2 = env
        .account_manager
        .create_account(
            "user2@localhost".to_string(),
            None,
            "greenmail".to_string(),
            env.get_imap_host(),
            env.imap_port,
            env.get_smtp_host(),
            env.smtp_port,
            AuthType::Password,
            Some("pass2".to_string()),
        )
        .await
        .unwrap();

    // List accounts
    let accounts = env.account_manager.list_accounts().await.unwrap();
    assert_eq!(accounts.len(), 2);

    // Cleanup
    env.account_manager.delete_account(&account1.id).await.unwrap();
    env.account_manager.delete_account(&account2.id).await.unwrap();
}

#[tokio::test]
async fn test_imap_connection() {
    let env = TestEnvironment::new().await;

    // Greenmail accepts any username/password
    let mut client = ImapTestClient::connect_plain(
        &env.get_imap_host(),
        env.imap_port,
        "test@localhost",
        "test",
    )
    .await
    .unwrap();

    // List mailboxes
    let mailboxes = client.list_mailboxes().await.unwrap();
    assert!(!mailboxes.is_empty());

    // Should have at least INBOX
    let has_inbox = mailboxes.iter().any(|m| m.path == "INBOX");
    assert!(has_inbox, "INBOX mailbox should exist");

    client.logout().await.unwrap();
}

#[tokio::test]
async fn test_smtp_send_email() {
    let env = TestEnvironment::new().await;

    // Create SMTP client (plain, no TLS for testing)
    let client = SmtpTestClient::new_plain(
        &env.get_smtp_host(),
        env.smtp_port,
        "sender@localhost",
        Some("Sender Name"),
    )
    .unwrap();

    // Create email
    let mut email = EmailData::new("Test Subject".to_string());
    email.add_to("recipient@localhost", Some("Recipient Name")).unwrap();
    email = email.with_text_body("This is a test email body.".to_string());

    // Send email
    let result = client.send_email(email);
    assert!(result.is_ok(), "Email should be sent successfully");
}

#[tokio::test]
async fn test_imap_send_and_receive() {
    let env = TestEnvironment::new().await;

    let test_email = "testuser@localhost";
    let test_password = "password";

    // Send an email via SMTP
    let smtp_client = SmtpTestClient::new_plain(
        &env.get_smtp_host(),
        env.smtp_port,
        test_email,
        Some("Test User"),
    )
    .unwrap();

    let mut email = EmailData::new("Integration Test Email".to_string());
    email.add_to(test_email, None).unwrap();
    email = email.with_text_body("This email tests the full send and receive flow.".to_string());

    smtp_client.send_email(email).unwrap();

    // Wait for email to be delivered
    sleep(Duration::from_secs(2)).await;

    // Connect via IMAP and fetch the email
    let mut imap_client = ImapTestClient::connect_plain(
        &env.get_imap_host(),
        env.imap_port,
        test_email,
        test_password,
    )
    .await
    .unwrap();

    // Select INBOX
    let mailbox = imap_client.select_mailbox("INBOX").await.unwrap();
    assert!(mailbox.exists > 0, "Should have at least one message in INBOX");

    // Fetch messages
    let messages = imap_client.fetch_messages("1:*").await.unwrap();
    assert!(!messages.is_empty(), "Should fetch at least one message");

    let message = &messages[0];
    assert!(message.envelope.is_some(), "Message should have envelope");

    let envelope = message.envelope.as_ref().unwrap();
    assert_eq!(
        envelope.subject,
        Some("Integration Test Email".to_string()),
        "Subject should match"
    );

    imap_client.logout().await.unwrap();
}

#[tokio::test]
async fn test_imap_mark_operations() {
    let env = TestEnvironment::new().await;

    let test_email = "marktest@localhost";
    let test_password = "password";

    // Send a test email
    let smtp_client = SmtpTestClient::new_plain(
        &env.get_smtp_host(),
        env.smtp_port,
        test_email,
        None,
    )
    .unwrap();

    let mut email = EmailData::new("Mark Test".to_string());
    email.add_to(test_email, None).unwrap();
    email = email.with_text_body("Testing mark operations.".to_string());
    smtp_client.send_email(email).unwrap();

    sleep(Duration::from_secs(2)).await;

    // Connect and test mark operations
    let mut imap_client = ImapTestClient::connect_plain(
        &env.get_imap_host(),
        env.imap_port,
        test_email,
        test_password,
    )
    .await
    .unwrap();

    imap_client.select_mailbox("INBOX").await.unwrap();

    let messages = imap_client.fetch_messages("1:*").await.unwrap();
    let uid = messages[0].uid;

    // Test mark as seen
    imap_client.mark_seen(&[uid]).await.unwrap();

    // Test mark as flagged/starred
    imap_client.mark_flagged(&[uid]).await.unwrap();

    // Test mark as unseen
    imap_client.mark_unseen(&[uid]).await.unwrap();

    // Test mark as unflagged
    imap_client.mark_unflagged(&[uid]).await.unwrap();

    imap_client.logout().await.unwrap();
}

#[tokio::test]
async fn test_full_sync_flow() {
    let env = TestEnvironment::new().await;

    let test_email = "synctest@localhost";
    let test_password = "password";

    // Create account
    let account = env
        .account_manager
        .create_account(
            test_email.to_string(),
            Some("Sync Test User".to_string()),
            "greenmail".to_string(),
            env.get_imap_host(),
            env.imap_port,
            env.get_smtp_host(),
            env.smtp_port,
            AuthType::Password,
            Some(test_password.to_string()),
        )
        .await
        .unwrap();

    // Send some test emails
    let smtp_client = SmtpTestClient::new_plain(
        &env.get_smtp_host(),
        env.smtp_port,
        test_email,
        Some("Sync Test"),
    )
    .unwrap();

    for i in 1..=3 {
        let mut email = EmailData::new(format!("Test Email {}", i));
        email.add_to(test_email, None).unwrap();
        email = email.with_text_body(format!("This is test email number {}", i));
        smtp_client.send_email(email).unwrap();
    }

    // Wait for emails to be delivered
    sleep(Duration::from_secs(3)).await;

    // Sync account
    let result = env.sync_service.sync_account(&account.id).await.unwrap();
    assert_eq!(result.new_messages, 3, "Should sync 3 new messages");

    // Verify emails are in database
    let emails: Vec<(String,)> = sqlx::query_as(
        "SELECT subject FROM emails WHERE account_id = ? ORDER BY date DESC",
    )
    .bind(&account.id)
    .fetch_all(env.db.pool())
    .await
    .unwrap();

    assert_eq!(emails.len(), 3, "Should have 3 emails in database");

    // Cleanup
    env.account_manager.delete_account(&account.id).await.unwrap();
}

#[tokio::test]
async fn test_mailbox_sync() {
    let env = TestEnvironment::new().await;

    let test_email = "mailboxtest@localhost";
    let test_password = "password";

    // Create account
    let account = env
        .account_manager
        .create_account(
            test_email.to_string(),
            None,
            "greenmail".to_string(),
            env.get_imap_host(),
            env.imap_port,
            env.get_smtp_host(),
            env.smtp_port,
            AuthType::Password,
            Some(test_password.to_string()),
        )
        .await
        .unwrap();

    // Sync to create mailboxes
    env.sync_service.sync_account(&account.id).await.unwrap();

    // Verify mailboxes are in database
    let mailboxes: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM mailboxes WHERE account_id = ?")
            .bind(&account.id)
            .fetch_all(env.db.pool())
            .await
            .unwrap();

    assert!(!mailboxes.is_empty(), "Should have mailboxes in database");

    // Should have INBOX
    let has_inbox = mailboxes.iter().any(|(name,)| name == "INBOX");
    assert!(has_inbox, "Should have INBOX mailbox");

    // Cleanup
    env.account_manager.delete_account(&account.id).await.unwrap();
}

#[tokio::test]
async fn test_incremental_sync() {
    let env = TestEnvironment::new().await;

    let test_email = "incrementaltest@localhost";
    let test_password = "password";

    // Create account
    let account = env
        .account_manager
        .create_account(
            test_email.to_string(),
            None,
            "greenmail".to_string(),
            env.get_imap_host(),
            env.imap_port,
            env.get_smtp_host(),
            env.smtp_port,
            AuthType::Password,
            Some(test_password.to_string()),
        )
        .await
        .unwrap();

    let smtp_client = SmtpTestClient::new_plain(
        &env.get_smtp_host(),
        env.smtp_port,
        test_email,
        None,
    )
    .unwrap();

    // Send initial email
    let mut email = EmailData::new("First Email".to_string());
    email.add_to(test_email, None).unwrap();
    email = email.with_text_body("First email content".to_string());
    smtp_client.send_email(email).unwrap();

    sleep(Duration::from_secs(2)).await;

    // First sync
    let result1 = env.sync_service.sync_account(&account.id).await.unwrap();
    assert_eq!(result1.new_messages, 1);

    // Send another email
    let mut email2 = EmailData::new("Second Email".to_string());
    email2.add_to(test_email, None).unwrap();
    email2 = email2.with_text_body("Second email content".to_string());
    smtp_client.send_email(email2).unwrap();

    sleep(Duration::from_secs(2)).await;

    // Second sync - should only fetch the new email
    let result2 = env.sync_service.sync_account(&account.id).await.unwrap();
    assert_eq!(result2.new_messages, 1, "Should only sync 1 new message");
    assert_eq!(result2.total_messages, 2, "Should have 2 total messages");

    // Cleanup
    env.account_manager.delete_account(&account.id).await.unwrap();
}
