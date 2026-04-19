//! Generic IMAP/SMTP runtime configuration and identifiers.

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportSecurity {
    Tls,
    StartTls,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    Password,
    OAuth2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GenericCredentials {
    Password {
        username: String,
        password: String,
    },
    OAuth2Bearer {
        username: String,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImapEndpoint {
    pub host: String,
    pub port: u16,
    pub security: TransportSecurity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmtpEndpoint {
    pub host: String,
    pub port: u16,
    pub security: TransportSecurity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericAccountConfig {
    pub email_address: String,
    pub display_name: Option<String>,
    pub imap: ImapEndpoint,
    pub smtp: SmtpEndpoint,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericMailbox {
    pub path: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImapEnvelopeSummary {
    pub uid: u32,
    pub message_id: Option<String>,
    pub subject: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImapMessageIdentity {
    pub mailbox_path: String,
    pub uidvalidity: u32,
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
        GenericAccountConfig, GenericCredentials, ImapEndpoint, ImapMessageIdentity, SmtpEndpoint,
        TransportSecurity,
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
}
