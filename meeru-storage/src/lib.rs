//! Meeru Storage - Data persistence layer
//!
//! This crate handles all data storage operations including SQLite database
//! management, file storage for email content, and search indexing.

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
    NewAttachment, NewEmail, NewFolderMapping, NewUnifiedFolder, ProviderType, UnifiedFolderRecord,
    UnifiedFolderType,
};
pub use store::{AccountStore, AttachmentStore, EmailStore, FolderStore};
