//! Generic IMAP/SMTP runtime configuration and identifiers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportSecurity {
    /// Open the connection inside a TLS session from the first byte.
    Tls,
    /// Negotiate TLS after connecting over a cleartext transport.
    StartTls,
    /// Leave the transport unencrypted.
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    /// Authenticate with a username and password.
    Password,
    /// Authenticate with a bearer token suitable for XOAUTH2-style login.
    OAuth2,
}

/// Runtime credentials supplied to the generic IMAP/SMTP adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GenericCredentials {
    Password {
        /// Login name presented to the remote IMAP/SMTP server.
        username: String,
        /// Secret used for classic password-based authentication.
        password: String,
    },
    OAuth2Bearer {
        /// Login name associated with the bearer token.
        username: String,
        /// Access token used for OAuth2 bearer authentication.
        access_token: String,
    },
}

impl GenericCredentials {
    pub fn auth_method(&self) -> AuthMethod {
        match self {
            Self::Password { .. } => AuthMethod::Password,
            Self::OAuth2Bearer { .. } => AuthMethod::OAuth2,
        }
    }

    pub fn username(&self) -> &str {
        match self {
            Self::Password { username, .. } | Self::OAuth2Bearer { username, .. } => username,
        }
    }
}

/// Connection settings for the account's IMAP endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImapEndpoint {
    /// Hostname or IP address of the IMAP server.
    pub host: String,
    /// TCP port used for the IMAP connection.
    pub port: u16,
    /// Transport security mode expected by the server.
    pub security: TransportSecurity,
}

/// Connection settings for the account's SMTP endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmtpEndpoint {
    /// Hostname or IP address of the SMTP server.
    pub host: String,
    /// TCP port used for the SMTP connection.
    pub port: u16,
    /// Transport security mode expected by the server.
    pub security: TransportSecurity,
}

/// Minimal runtime configuration required to talk to a generic IMAP/SMTP account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericAccountConfig {
    /// Primary mailbox address used as the account identity.
    pub email_address: String,
    /// Optional display name used when composing outgoing mail.
    pub display_name: Option<String>,
    /// Connection settings for message retrieval and folder discovery.
    pub imap: ImapEndpoint,
    /// Connection settings for message submission.
    pub smtp: SmtpEndpoint,
    /// Credentials shared by the IMAP and SMTP transports.
    pub credentials: GenericCredentials,
}

impl GenericAccountConfig {
    pub fn validate(&self) -> Result<()> {
        if self.email_address.trim().is_empty() {
            return Err(Error::InvalidConfiguration(
                "email address must not be empty".to_string(),
            ));
        }

        validate_endpoint("imap", &self.imap.host, self.imap.port)?;
        validate_endpoint("smtp", &self.smtp.host, self.smtp.port)?;

        if self.credentials.username().trim().is_empty() {
            return Err(Error::InvalidConfiguration(
                "credentials username must not be empty".to_string(),
            ));
        }

        match &self.credentials {
            GenericCredentials::Password { password, .. } if password.is_empty() => {
                Err(Error::InvalidConfiguration(
                    "password credentials must include a password".to_string(),
                ))
            },
            GenericCredentials::OAuth2Bearer { access_token, .. } if access_token.is_empty() => {
                Err(Error::InvalidConfiguration(
                    "oauth2 credentials must include an access token".to_string(),
                ))
            },
            _ => Ok(()),
        }
    }
}

/// Remote mailbox discovered from the IMAP server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericMailbox {
    /// Provider-native mailbox path used in IMAP commands and sync identity.
    pub path: String,
    /// Human-readable mailbox label suitable for UI display.
    pub display_name: String,
}

/// Lightweight summary of a message returned by mailbox listing and search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImapEnvelopeSummary {
    /// IMAP UID scoped to the mailbox identified by the fetch request.
    pub uid: u32,
    /// Stable local sync key derived from mailbox path, UIDVALIDITY, and UID.
    pub provider_id: String,
    /// RFC822 `Message-ID` header when present.
    pub message_id: Option<String>,
    /// Decoded subject line if the message exposes one.
    pub subject: Option<String>,
    /// Server-reported internal timestamp for the message.
    pub internal_date: Option<DateTime<Utc>>,
}

/// Mailbox-scoped IMAP identity used to build a durable local provider id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImapMessageIdentity {
    /// Provider mailbox path that owns the UID.
    pub mailbox_path: String,
    /// IMAP UIDVALIDITY value that scopes the lifetime of the UID.
    pub uidvalidity: u32,
    /// IMAP UID inside the mailbox and UIDVALIDITY pair.
    pub uid: u32,
}

impl ImapMessageIdentity {
    pub fn new(mailbox_path: impl Into<String>, uidvalidity: u32, uid: u32) -> Self {
        Self {
            mailbox_path: mailbox_path.into(),
            uidvalidity,
            uid,
        }
    }

    pub fn provider_id(&self) -> String {
        format!(
            "{}:{}:{}",
            escape_mailbox_path(&self.mailbox_path),
            self.uidvalidity,
            self.uid
        )
    }
}

/// Message payload fetched from IMAP for local parsing and storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedMessage {
    /// Mailbox-scoped identity of the fetched message.
    pub identity: ImapMessageIdentity,
    /// Canonical raw RFC822 bytes returned by the remote server.
    pub raw_message: Vec<u8>,
}

/// Normalized outgoing message payload accepted by the generic SMTP sender.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingMessage {
    /// Optional explicit From address; defaults to the account address when omitted.
    pub from: Option<String>,
    /// Optional Reply-To header for responses.
    pub reply_to: Option<String>,
    /// Recipient addresses rendered into the SMTP envelope and message headers.
    pub to: Vec<String>,
    /// Message subject line.
    pub subject: String,
    /// Required plain-text body for the MVP send path.
    pub text_body: String,
    /// Optional HTML alternative body part.
    pub html_body: Option<String>,
}

impl OutgoingMessage {
    pub fn validate(&self) -> Result<()> {
        if self.to.is_empty() {
            return Err(Error::InvalidConfiguration(
                "outgoing message must include at least one recipient".to_string(),
            ));
        }

        if self.subject.trim().is_empty() {
            return Err(Error::InvalidConfiguration(
                "outgoing message subject must not be empty".to_string(),
            ));
        }

        if self.text_body.is_empty() {
            return Err(Error::InvalidConfiguration(
                "outgoing message text body must not be empty".to_string(),
            ));
        }

        Ok(())
    }
}

fn validate_endpoint(kind: &str, host: &str, port: u16) -> Result<()> {
    if host.trim().is_empty() {
        return Err(Error::InvalidConfiguration(format!(
            "{kind} host must not be empty"
        )));
    }

    if port == 0 {
        return Err(Error::InvalidConfiguration(format!(
            "{kind} port must be greater than zero"
        )));
    }

    Ok(())
}

fn escape_mailbox_path(path: &str) -> String {
    path.replace('\\', "\\\\").replace(':', "\\:")
}

#[cfg(test)]
mod tests {
    use super::{
        GenericAccountConfig, GenericCredentials, ImapEndpoint, ImapMessageIdentity,
        OutgoingMessage, SmtpEndpoint, TransportSecurity,
    };

    #[test]
    fn validates_generic_account_config() {
        let config = GenericAccountConfig {
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
        };

        config.validate().expect("config should be valid");
    }

    #[test]
    fn rejects_empty_credentials() {
        let config = GenericAccountConfig {
            email_address: "alice@example.com".to_string(),
            display_name: None,
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
                password: String::new(),
            },
        };

        let error = config.validate().expect_err("config should be invalid");
        assert!(error.to_string().contains("password"));
    }

    #[test]
    fn builds_mailbox_scoped_provider_id() {
        let identity = ImapMessageIdentity::new("Archive:2026", 42, 7);

        assert_eq!(identity.provider_id(), "Archive\\:2026:42:7");
    }

    #[test]
    fn validates_outgoing_message() {
        let message = OutgoingMessage {
            from: None,
            reply_to: None,
            to: vec!["bob@example.com".to_string()],
            subject: "Hello".to_string(),
            text_body: "Hello, world!".to_string(),
            html_body: Some("<p>Hello, world!</p>".to_string()),
        };

        message.validate().expect("message should be valid");
    }
}
