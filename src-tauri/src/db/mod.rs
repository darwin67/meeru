pub mod imap;
pub mod model;

use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

pub struct Database {
    // pool is the connecton pool used for sqlite
    pool: SqlitePool,
}

impl Database {
    pub async fn new(path: PathBuf) -> Result<Self> {
        let db_url = format!("sqlite:{}", path.display());

        let opts = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_secs(30));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await
            .context("Failed to connect to database")?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!()
            .run(&self.pool)
            .await
            .context("Failed to run database migrations")?;
        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path).await.unwrap();

        // run migrations
        db.migrate().await.unwrap();

        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_db_creation() {
        let (db, _tmpdir) = create_test_db().await;

        // Verify tables exist by querying sqlite_master
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('accounts', 'account_endpoints', 'folders', 'messages', 'folder_messages', 'imap_folders_state', 'imap_message_state', 'pop3_state', 'pop3_message_state', 'jmap_state', 'jmap_message_state', 'eas_folder_state', 'eas_message_state')",
        )
        .fetch_one(db.pool())
        .await
        .unwrap();
        assert_eq!(result.0, 12, "Expected tables to be created");
    }
}
