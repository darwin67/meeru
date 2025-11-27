use crate::db::Database;

use anyhow::Result;

use std::path::PathBuf;

pub struct State {
    pub db: Database,
}

impl State {
    pub async fn new() -> Result<Self> {
        // TODO provide config and reference that path instead
        let path = PathBuf::from("meeru.db");
        let db = Database::new(path).await?;

        Ok(Self { db })
    }
}
