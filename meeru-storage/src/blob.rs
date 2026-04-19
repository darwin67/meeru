//! File-system blob storage for email bodies and attachment payloads.

use std::path::{Component, Path, PathBuf};

use async_trait::async_trait;
use tokio::fs;
use uuid::Uuid;

use crate::{Error, Result, Storage};

#[async_trait]
pub trait BlobStore {
    async fn put_email_body(&self, email_id: Uuid, content: &[u8]) -> Result<String>;
    async fn put_attachment_payload(
        &self,
        attachment_id: Uuid,
        filename: &str,
        content: &[u8],
    ) -> Result<String>;
    async fn read_blob(&self, relative_path: &str) -> Result<Vec<u8>>;
    async fn delete_blob(&self, relative_path: &str) -> Result<()>;
}

#[async_trait]
impl BlobStore for Storage {
    async fn put_email_body(&self, email_id: Uuid, content: &[u8]) -> Result<String> {
        let relative_path = format!("blobs/emails/{email_id}.eml");
        self.write_blob(&relative_path, content).await?;
        Ok(relative_path)
    }

    async fn put_attachment_payload(
        &self,
        attachment_id: Uuid,
        filename: &str,
        content: &[u8],
    ) -> Result<String> {
        let relative_path = format!(
            "blobs/attachments/{attachment_id}-{}",
            sanitize_filename(filename)
        );
        self.write_blob(&relative_path, content).await?;
        Ok(relative_path)
    }

    async fn read_blob(&self, relative_path: &str) -> Result<Vec<u8>> {
        let absolute_path = self.resolve_relative_blob_path(relative_path)?;
        let bytes = fs::read(&absolute_path).await.map_err(map_read_error)?;
        Ok(bytes)
    }

    async fn delete_blob(&self, relative_path: &str) -> Result<()> {
        let absolute_path = self.resolve_relative_blob_path(relative_path)?;

        match fs::remove_file(&absolute_path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Err(Error::NotFound(format!("blob {relative_path}")))
            },
            Err(error) => Err(Error::Io(error)),
        }
    }
}

impl Storage {
    async fn write_blob(&self, relative_path: &str, content: &[u8]) -> Result<()> {
        let absolute_path = self.resolve_relative_blob_path(relative_path)?;
        let parent = absolute_path.parent().ok_or_else(|| {
            Error::InvalidPath(format!(
                "blob path has no parent directory: {relative_path}"
            ))
        })?;

        fs::create_dir_all(parent).await?;

        let temp_path = self.paths().temp.join(format!("{}.tmp", Uuid::new_v4()));
        fs::write(&temp_path, content).await?;

        let write_result = fs::rename(&temp_path, &absolute_path).await;
        if let Err(error) = write_result {
            let _ = fs::remove_file(&temp_path).await;
            return Err(Error::Io(error));
        }

        Ok(())
    }

    fn resolve_relative_blob_path(&self, relative_path: &str) -> Result<PathBuf> {
        validate_relative_path(relative_path)?;
        Ok(self.paths().root.join(relative_path))
    }
}

fn sanitize_filename(filename: &str) -> String {
    let sanitized: String = filename
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => ch,
            _ => '_',
        })
        .collect();

    if sanitized.is_empty() {
        "attachment.bin".to_string()
    } else {
        sanitized
    }
}

fn validate_relative_path(relative_path: &str) -> Result<()> {
    let path = Path::new(relative_path);
    if path.is_absolute() {
        return Err(Error::InvalidPath(format!(
            "absolute blob paths are not allowed: {relative_path}"
        )));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {},
            Component::CurDir => {},
            Component::ParentDir => {
                return Err(Error::InvalidPath(format!(
                    "parent traversal is not allowed in blob paths: {relative_path}"
                )));
            },
            _ => {
                return Err(Error::InvalidPath(format!(
                    "unsupported blob path component in: {relative_path}"
                )));
            },
        }
    }

    Ok(())
}

fn map_read_error(error: std::io::Error) -> Error {
    if error.kind() == std::io::ErrorKind::NotFound {
        Error::NotFound("blob".to_string())
    } else {
        Error::Io(error)
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_filename;

    #[test]
    fn sanitizes_attachment_filenames() {
        assert_eq!(sanitize_filename("invoice 2026?.pdf"), "invoice_2026_.pdf");
        assert_eq!(sanitize_filename(""), "attachment.bin");
    }
}
