//! Concrete generic IMAP/SMTP adapter for the MVP provider slice.

use async_trait::async_trait;

use crate::{
    generic::{
        FetchedMessage, GenericAccountConfig, GenericMailbox, ImapEnvelopeSummary, OutgoingMessage,
    },
    imap, smtp, EmailProvider, ProviderCapabilities, Result,
};

/// Concrete provider implementation that speaks directly to generic IMAP and SMTP servers.
#[derive(Debug, Clone)]
pub struct GenericImapSmtpAdapter {
    /// Validated runtime configuration shared by IMAP and SMTP operations.
    config: GenericAccountConfig,
    /// Lightweight connection flag used by the provider trait surface.
    connected: bool,
}

impl GenericImapSmtpAdapter {
    /// Build an adapter from runtime configuration after validating required fields.
    pub fn new(config: GenericAccountConfig) -> Result<Self> {
        config.validate()?;

        Ok(Self {
            config,
            connected: false,
        })
    }

    /// Return the validated runtime configuration backing this adapter.
    pub fn config(&self) -> &GenericAccountConfig {
        &self.config
    }

    /// List remote IMAP mailboxes that can be synced or mapped into unified folders.
    pub async fn list_mailboxes(&self) -> Result<Vec<GenericMailbox>> {
        imap::list_mailboxes(&self.config).await
    }

    /// Fetch the newest message summaries from a mailbox for preview and sync selection.
    pub async fn fetch_mailbox_page(
        &self,
        mailbox_path: &str,
        page_size: usize,
    ) -> Result<Vec<ImapEnvelopeSummary>> {
        imap::fetch_mailbox_page(&self.config, mailbox_path, page_size).await
    }

    /// Run a provider-side mailbox search and return lightweight message summaries.
    pub async fn search_mailbox(
        &self,
        mailbox_path: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ImapEnvelopeSummary>> {
        imap::search_mailbox(&self.config, mailbox_path, query, limit).await
    }

    /// Fetch raw RFC822 payloads for specific UIDs in a mailbox.
    pub async fn fetch_messages(
        &self,
        mailbox_path: &str,
        uids: &[u32],
    ) -> Result<Vec<FetchedMessage>> {
        imap::fetch_messages(&self.config, mailbox_path, uids).await
    }

    /// Submit a normalized outgoing message through the configured SMTP endpoint.
    pub async fn send_message(&self, message: OutgoingMessage) -> Result<()> {
        smtp::send_message(&self.config, &message).await
    }
}

#[async_trait]
impl EmailProvider for GenericImapSmtpAdapter {
    async fn connect(&mut self) -> Result<()> {
        imap::validate_connectivity(&self.config).await?;
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_folders: true,
            supports_labels: false,
            supports_search: true,
            supports_push: false,
            supports_oauth: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        traits::EmailProvider, AuthMethod, GenericAccountConfig, GenericCredentials,
        GenericImapSmtpAdapter, ImapEndpoint, SmtpEndpoint, TransportSecurity,
    };

    fn sample_config() -> GenericAccountConfig {
        GenericAccountConfig {
            email_address: "alice@example.com".to_string(),
            display_name: Some("Alice".to_string()),
            imap: ImapEndpoint {
                host: "imap.example.com".to_string(),
                port: 993,
                security: TransportSecurity::Tls,
            },
            smtp: SmtpEndpoint {
                host: "smtp.example.com".to_string(),
                port: 465,
                security: TransportSecurity::Tls,
            },
            credentials: GenericCredentials::Password {
                username: "alice@example.com".to_string(),
                password: "secret".to_string(),
            },
        }
    }

    #[test]
    fn adapter_construction_validates_config() {
        let adapter = GenericImapSmtpAdapter::new(sample_config()).expect("adapter should build");

        assert_eq!(
            adapter.config().credentials.auth_method(),
            AuthMethod::Password
        );
    }

    #[test]
    fn adapter_reports_generic_imap_capabilities() {
        let adapter = GenericImapSmtpAdapter::new(sample_config()).expect("adapter should build");
        let capabilities = adapter.capabilities();

        assert!(capabilities.supports_folders);
        assert!(capabilities.supports_search);
        assert!(capabilities.supports_oauth);
        assert!(!capabilities.supports_labels);
        assert!(!capabilities.supports_push);
    }
}
