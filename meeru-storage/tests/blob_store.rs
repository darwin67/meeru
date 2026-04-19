use meeru_storage::{BlobStore, StorageConfig};
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::test]
async fn email_body_blobs_round_trip_and_delete() {
    let temp_dir = TempDir::new().expect("temp dir");
    let storage = StorageConfig::new(temp_dir.path())
        .open()
        .await
        .expect("open storage");

    let relative_path = storage
        .put_email_body(Uuid::new_v4(), b"Subject: Test\r\n\r\nHello")
        .await
        .expect("write email body");
    let bytes = storage
        .read_blob(&relative_path)
        .await
        .expect("read email body");

    assert_eq!(bytes, b"Subject: Test\r\n\r\nHello");
    assert!(storage.paths().root.join(&relative_path).exists());

    storage
        .delete_blob(&relative_path)
        .await
        .expect("delete email body");
    assert!(!storage.paths().root.join(&relative_path).exists());
}

#[tokio::test]
async fn attachment_payloads_get_deterministic_relative_paths() {
    let temp_dir = TempDir::new().expect("temp dir");
    let storage = StorageConfig::new(temp_dir.path())
        .open()
        .await
        .expect("open storage");
    let attachment_id = Uuid::new_v4();

    let relative_path = storage
        .put_attachment_payload(attachment_id, "invoice 2026?.pdf", b"pdf-bytes")
        .await
        .expect("write attachment");
    let bytes = storage
        .read_blob(&relative_path)
        .await
        .expect("read attachment");

    assert_eq!(
        relative_path,
        format!("blobs/attachments/{attachment_id}-invoice_2026_.pdf")
    );
    assert_eq!(bytes, b"pdf-bytes");
}
