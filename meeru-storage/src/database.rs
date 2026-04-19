//! SQLite database bootstrap and storage path management.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use directories::ProjectDirs;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};

use crate::{migrations, Error, Result};

const DEFAULT_DATABASE_FILE: &str = "meeru.db";
const DEFAULT_MAX_CONNECTIONS: u32 = 5;

/// Stable storage paths for the local Meeru data root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoragePaths {
    pub root: PathBuf,
    pub database: PathBuf,
    pub blobs: PathBuf,
    pub email_bodies: PathBuf,
    pub attachments: PathBuf,
    pub temp: PathBuf,
    pub backups: PathBuf,
}

impl StoragePaths {
    /// Build the directory layout rooted at the provided path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let blobs = root.join("blobs");

        Self {
            database: root.join(DEFAULT_DATABASE_FILE),
            email_bodies: blobs.join("emails"),
            attachments: blobs.join("attachments"),
            temp: root.join("tmp"),
            backups: root.join("backups"),
            blobs,
            root,
        }
    }

    /// Directories that must exist before the storage runtime can open.
    pub fn required_directories(&self) -> [&Path; 6] {
        [
            self.root.as_path(),
            self.blobs.as_path(),
            self.email_bodies.as_path(),
            self.attachments.as_path(),
            self.temp.as_path(),
            self.backups.as_path(),
        ]
    }
}

/// Configuration used to open the Meeru local storage runtime.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    root: PathBuf,
    max_connections: u32,
}

impl StorageConfig {
    /// Create a storage configuration rooted at a specific path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            max_connections: DEFAULT_MAX_CONNECTIONS,
        }
    }

    /// Create a storage configuration using the OS-specific app data directory.
    pub fn from_project_dirs() -> Result<Self> {
        let project_dirs =
            ProjectDirs::from("io.github", "darwin67", "meeru").ok_or_else(|| {
                Error::Configuration(
                    "unable to determine platform-specific application data directory".to_string(),
                )
            })?;

        Ok(Self::new(project_dirs.data_local_dir()))
    }

    /// Override the SQLite pool size used by the storage runtime.
    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections.max(1);
        self
    }

    /// Resolve the storage paths for this configuration.
    pub fn paths(&self) -> StoragePaths {
        StoragePaths::new(self.root.clone())
    }

    /// Open the local storage runtime, creating directories and applying migrations.
    pub async fn open(&self) -> Result<Storage> {
        let storage = self.open_without_migrations().await?;
        migrations::run_migrations(storage.pool()).await?;

        Ok(storage)
    }

    /// Open the local storage runtime without applying migrations.
    pub async fn open_without_migrations(&self) -> Result<Storage> {
        let paths = self.paths();
        create_required_directories(&paths).await?;

        let options = SqliteConnectOptions::new()
            .filename(&paths.database)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(self.max_connections)
            .acquire_timeout(Duration::from_secs(10))
            .connect_with(options)
            .await
            .map_err(|source| Error::OpenDatabase {
                path: paths.database.clone(),
                source,
            })?;

        Ok(Storage { pool, paths })
    }
}

/// Opened local storage runtime.
#[derive(Debug, Clone)]
pub struct Storage {
    pool: SqlitePool,
    paths: StoragePaths,
}

impl Storage {
    /// Access the SQLite connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Access the resolved on-disk storage paths.
    pub fn paths(&self) -> &StoragePaths {
        &self.paths
    }
}

async fn create_required_directories(paths: &StoragePaths) -> Result<()> {
    for directory in paths.required_directories() {
        std::fs::create_dir_all(directory).map_err(|source| Error::CreateDirectory {
            path: directory.to_path_buf(),
            source,
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::StoragePaths;
    use std::path::PathBuf;

    #[test]
    fn derives_expected_paths_from_root() {
        let paths = StoragePaths::new(PathBuf::from("/tmp/meeru-test"));

        assert_eq!(paths.database, PathBuf::from("/tmp/meeru-test/meeru.db"));
        assert_eq!(paths.blobs, PathBuf::from("/tmp/meeru-test/blobs"));
        assert_eq!(
            paths.email_bodies,
            PathBuf::from("/tmp/meeru-test/blobs/emails")
        );
        assert_eq!(
            paths.attachments,
            PathBuf::from("/tmp/meeru-test/blobs/attachments")
        );
        assert_eq!(paths.temp, PathBuf::from("/tmp/meeru-test/tmp"));
        assert_eq!(paths.backups, PathBuf::from("/tmp/meeru-test/backups"));
    }
}
