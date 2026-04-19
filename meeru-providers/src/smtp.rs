//! SMTP operations for the generic provider baseline.

use lettre::{
    message::{Mailbox, MultiPart},
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::{
    generic::{GenericAccountConfig, GenericCredentials, OutgoingMessage},
    Error, Result, TransportSecurity,
};

pub async fn send_message(config: &GenericAccountConfig, message: &OutgoingMessage) -> Result<()> {
    message.validate()?;
    let email = build_message(config, message)?;
    let transport = build_transport(config)?;

    transport
        .send(email)
        .await
        .map_err(|error| Error::Connection(error.to_string()))?;

    Ok(())
}

fn build_message(config: &GenericAccountConfig, message: &OutgoingMessage) -> Result<Message> {
    let from = message
        .from
        .as_deref()
        .map(parse_mailbox)
        .transpose()?
        .unwrap_or_else(|| default_sender(config));
    let mut builder = Message::builder().from(from).subject(message.subject.clone());

    if let Some(reply_to) = message.reply_to.as_deref() {
        builder = builder.reply_to(parse_mailbox(reply_to)?);
    }

    for recipient in &message.to {
        builder = builder.to(parse_mailbox(recipient)?);
    }

    if let Some(html_body) = &message.html_body {
        builder
            .multipart(MultiPart::alternative_plain_html(
                message.text_body.clone(),
                html_body.clone(),
            ))
            .map_err(|error| Error::Other(error.to_string()))
    } else {
        builder
            .body(message.text_body.clone())
            .map_err(|error| Error::Other(error.to_string()))
    }
}

fn build_transport(config: &GenericAccountConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    let builder = match config.smtp.security {
        TransportSecurity::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp.host)
            .map_err(|error| Error::Connection(error.to_string()))?
            .port(config.smtp.port),
        TransportSecurity::StartTls => {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp.host)
                .map_err(|error| Error::Connection(error.to_string()))?
                .port(config.smtp.port)
        },
        TransportSecurity::None => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
            &config.smtp.host,
        )
        .port(config.smtp.port),
    };

    let builder = match &config.credentials {
        GenericCredentials::Password { username, password } => builder
            .credentials(Credentials::new(username.clone(), password.clone()))
            .authentication(vec![Mechanism::Plain]),
        GenericCredentials::OAuth2Bearer {
            username,
            access_token,
        } => builder
            .credentials(Credentials::new(username.clone(), access_token.clone()))
            .authentication(vec![Mechanism::Xoauth2]),
    };

    Ok(builder.build())
}

fn default_sender(config: &GenericAccountConfig) -> Mailbox {
    Mailbox::new(
        config.display_name.clone(),
        config.email_address.parse().expect("validated email address"),
    )
}

fn parse_mailbox(value: &str) -> Result<Mailbox> {
    value
        .parse()
        .map_err(|error| Error::InvalidConfiguration(format!("invalid mailbox {value}: {error}")))
}

#[cfg(test)]
mod tests {
    use crate::{
        GenericAccountConfig, GenericCredentials, ImapEndpoint, OutgoingMessage, SmtpEndpoint,
        TransportSecurity,
    };

    use super::build_message;

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
    fn builds_plain_text_messages() {
        let message = OutgoingMessage {
            from: None,
            reply_to: None,
            to: vec!["bob@example.com".to_string()],
            subject: "Hello".to_string(),
            text_body: "Hello from text".to_string(),
            html_body: None,
        };

        let email = build_message(&sample_config(), &message).expect("message should build");
        let formatted = String::from_utf8(email.formatted()).expect("valid utf8");

        assert!(formatted.contains("Subject: Hello"));
        assert!(formatted.contains("Hello from text"));
    }

    #[test]
    fn builds_alternative_text_html_messages() {
        let message = OutgoingMessage {
            from: None,
            reply_to: Some("reply@example.com".to_string()),
            to: vec!["bob@example.com".to_string()],
            subject: "Hello".to_string(),
            text_body: "Hello from text".to_string(),
            html_body: Some("<p>Hello from html</p>".to_string()),
        };

        let email = build_message(&sample_config(), &message).expect("message should build");
        let formatted = String::from_utf8(email.formatted()).expect("valid utf8");

        assert!(formatted.contains("Reply-To:"));
        assert!(formatted.contains("reply@example.com"));
        assert!(formatted.contains("multipart/alternative"));
        assert!(formatted.contains("Hello from text"));
        assert!(formatted.contains("<p>Hello from html</p>"));
    }
}
