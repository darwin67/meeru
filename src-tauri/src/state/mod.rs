use crate::db::Database;

use anyhow::Result;

use std::path::PathBuf;

pub struct AppState {
    pub db: Database,
}

impl AppState {
    pub async fn new(app_data_dir: PathBuf) -> Result<Self> {
        let db_path = app_data_dir.join("meeru.db");
        let db = Database::new(db_path).await?;

        Ok(Self { db })
    }
}

// TODO add tests, right now it's redundant
