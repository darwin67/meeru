//! Provider trait definitions

use crate::Result;
use async_trait::async_trait;

#[async_trait]
pub trait EmailProvider {
    /// Connect to the email provider
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the provider
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Get provider capabilities
    fn capabilities(&self) -> ProviderCapabilities;
}

#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    pub supports_folders: bool,
    pub supports_labels: bool,
    pub supports_search: bool,
    pub supports_push: bool,
    pub supports_oauth: bool,
}
