//! Meeru Storage - Data persistence layer
//!
//! This crate handles all data storage operations including SQLite database
//! management, file storage for email content, and search indexing.

pub mod database;
pub mod error;
pub mod migrations;

pub use database::{Storage, StorageConfig, StoragePaths};
pub use error::{Error, Result};
