//! Meeru Storage - Data persistence layer
//!
//! This crate handles all data storage operations including SQLite database
//! management, file storage for email content, and search indexing.
//!
//! Basic bootstrap example:
//! ```no_run
//! use meeru_storage::{AccountStore, StorageConfig};
//!
//! # async fn example() -> meeru_storage::Result<()> {
//! let storage = StorageConfig::from_project_dirs()?.open().await?;
//! let _accounts = storage.list_accounts().await?;
//! # Ok(())
//! # }
//! ```

pub mod blob;
pub mod database;
pub mod error;
pub mod migrations;
pub mod model;
pub mod store;

pub use blob::BlobStore;
pub use database::{Storage, StorageConfig, StoragePaths};
pub use error::{Error, Result};
pub use model::{
    AccountRecord, AttachmentRecord, EmailAddress, EmailRecord, FolderMappingRecord, NewAccount,
    NewAttachment, NewEmail, NewEmailBundle, NewFolderMapping, NewUnifiedFolder, ProviderType,
    UnifiedFolderRecord, UnifiedFolderType,
};
pub use store::{AccountStore, AttachmentStore, EmailStore, FolderStore};
