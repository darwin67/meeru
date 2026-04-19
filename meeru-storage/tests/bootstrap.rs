use meeru_storage::{migrations, Error, StorageConfig};
use tempfile::TempDir;

#[tokio::test]
async fn bootstrap_creates_directories_and_applies_migrations() {
    let temp_dir = TempDir::new().expect("temp dir");
    let config = StorageConfig::new(temp_dir.path());
    let storage = config.open().await.expect("open storage");

    let paths = storage.paths();
    assert!(paths.root.exists());
    assert!(paths.database.exists());
    assert!(paths.blobs.exists());
    assert!(paths.email_bodies.exists());
    assert!(paths.attachments.exists());
    assert!(paths.temp.exists());

    let applied_versions = migrations::applied_versions(storage.pool())
        .await
        .expect("migration versions");
    assert_eq!(
        applied_versions,
        vec![0, migrations::current_schema_version()]
    );
}

#[tokio::test]
async fn reopen_preserves_existing_data() {
    let temp_dir = TempDir::new().expect("temp dir");
    let config = StorageConfig::new(temp_dir.path());

    let storage = config.open().await.expect("open storage");
    sqlx::query(
        r#"
INSERT INTO accounts (id, email, display_name, provider_type)
VALUES (?, ?, ?, ?)
        "#,
    )
    .bind("account-1")
    .bind("alice@example.com")
    .bind("Alice")
    .bind("generic")
    .execute(storage.pool())
    .await
    .expect("insert account");
    drop(storage);

    let reopened = config.open().await.expect("reopen storage");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts")
        .fetch_one(reopened.pool())
        .await
        .expect("count accounts");

    assert_eq!(count, 1);
}

#[tokio::test]
async fn rerunning_migrations_is_a_noop() {
    let temp_dir = TempDir::new().expect("temp dir");
    let storage = StorageConfig::new(temp_dir.path())
        .open()
        .await
        .expect("open storage");

    let applied = migrations::run_migrations(storage.pool())
        .await
        .expect("rerun migrations");
    let applied_versions = migrations::applied_versions(storage.pool())
        .await
        .expect("migration versions");

    assert!(applied.is_empty());
    assert_eq!(
        applied_versions,
        vec![0, migrations::current_schema_version()]
    );
}

#[tokio::test]
async fn bootstrap_directory_failures_include_the_failing_path() {
    let temp_dir = TempDir::new().expect("temp dir");
    let file_root = temp_dir.path().join("not-a-directory");
    std::fs::write(&file_root, "blocking file").expect("write blocking file");

    let error = StorageConfig::new(&file_root)
        .open()
        .await
        .expect_err("bootstrap should fail");

    match error {
        Error::CreateDirectory { path, .. } => assert_eq!(path, file_root),
        other => panic!("expected CreateDirectory error, got {other:?}"),
    }
}
