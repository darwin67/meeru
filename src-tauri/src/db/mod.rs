use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::log::LevelFilter;

pub mod models;

/// Database connection pool
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create database directory")?;
        }

        let db_url = format!("sqlite:{}", db_path.display());

        let mut options = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_secs(30));

        // Disable sqlx query logging to avoid noise
        options.log_statements(LevelFilter::Off);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("Failed to connect to database")?;

        let db = Self { pool };

        // Run migrations
        db.run_migrations().await?;

        Ok(db)
    }

    /// Run database migrations
    async fn run_migrations(&self) -> Result<()> {
        // Read and execute migration file
        let migration_sql = include_str!("migrations/001_initial.sql");

        sqlx::query(migration_sql)
            .execute(&self.pool)
            .await
            .context("Failed to run migrations")?;

        Ok(())
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Close the database connection
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
        (db, temp_dir)
    }

    #[tokio::test]
    async fn test_database_creation() {
        let (db, _temp_dir) = create_test_db().await;

        // Verify tables exist by querying sqlite_master
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('accounts', 'emails', 'threads', 'mailboxes', 'contacts')"
        )
        .fetch_one(db.pool())
        .await
        .unwrap();

        assert_eq!(result.0, 5, "Expected 5 main tables to be created");
    }

    #[tokio::test]
    async fn test_foreign_keys_enabled() {
        let (db, _temp_dir) = create_test_db().await;

        let result: (i64,) = sqlx::query_as("PRAGMA foreign_keys")
            .fetch_one(db.pool())
            .await
            .unwrap();

        assert_eq!(result.0, 1, "Foreign keys should be enabled");
    }
}
