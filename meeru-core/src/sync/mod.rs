//! Email synchronization engine

use crate::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SyncEngine {
    async fn sync_account(&self, account_id: uuid::Uuid) -> Result<SyncReport>;
    async fn sync_all_accounts(&self) -> Result<Vec<SyncReport>>;
}

#[derive(Debug)]
pub struct SyncReport {
    pub account_id: uuid::Uuid,
    pub emails_fetched: usize,
    pub emails_updated: usize,
    pub errors: Vec<String>,
}
