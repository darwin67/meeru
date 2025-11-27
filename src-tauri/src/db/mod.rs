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

        Ok(Self { pool: pool })
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
