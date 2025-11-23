use anyhow::{Context, Result};
use lettre::message::{header::ContentType, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub struct SmtpClient {
    transport: SmtpTransport,
    from_address: Mailbox,
}

impl SmtpClient {
    /// Create a new SMTP client
    pub fn new(host: &str, port: u16, email: &str, password: &str, from_name: Option<&str>) -> Result<Self> {
        let creds = Credentials::new(email.to_string(), password.to_string());

        let transport = SmtpTransport::starttls_relay(host)
            .context("Failed to create SMTP transport")?
            .port(port)
            .credentials(creds)
            .build();

        let from_address = if let Some(name) = from_name {
            Mailbox::new(Some(name.to_string()), email.parse().context("Invalid from email")?)
        } else {
            Mailbox::new(None, email.parse().context("Invalid from email")?)
        };

        Ok(Self {
            transport,
            from_address,
        })
    }

    /// Send an email
    pub fn send_email(&self, email_data: EmailData) -> Result<()> {
        let mut message_builder = Message::builder()
            .from(self.from_address.clone());

        // Add To recipients
        for to in &email_data.to {
            message_builder = message_builder.to(to.clone());
        }

        // Add Cc recipients
        for cc in &email_data.cc {
            message_builder = message_builder.cc(cc.clone());
        }

        // Add Bcc recipients
        for bcc in &email_data.bcc {
            message_builder = message_builder.bcc(bcc.clone());
        }

        // Add subject
        message_builder = message_builder.subject(&email_data.subject);

        // Add In-Reply-To and References headers if this is a reply
        if let Some(in_reply_to) = &email_data.in_reply_to {
            message_builder = message_builder.in_reply_to(in_reply_to.parse()?);
        }

        if let Some(references) = &email_data.references {
            for reference in references {
                message_builder = message_builder.references(reference.parse()?);
            }
        }

        // Build message body
        let message = if let Some(html) = &email_data.body_html {
            // If we have both plain text and HTML, use multipart alternative
            if let Some(text) = &email_data.body_text {
                message_builder.multipart(
                    MultiPart::alternative()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(text.clone()),
                        )
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html.clone()),
                        ),
                )?
            } else {
                // HTML only
                message_builder
                    .header(ContentType::TEXT_HTML)
                    .body(html.clone())?
            }
        } else if let Some(text) = &email_data.body_text {
            // Plain text only
            message_builder
                .header(ContentType::TEXT_PLAIN)
                .body(text.clone())?
        } else {
            anyhow::bail!("Email must have either text or HTML body");
        };

        // Send the email
        self.transport
            .send(&message)
            .context("Failed to send email")?;

        Ok(())
    }
}

/// Email data for sending
#[derive(Debug, Clone)]
pub struct EmailData {
    pub to: Vec<Mailbox>,
    pub cc: Vec<Mailbox>,
    pub bcc: Vec<Mailbox>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Option<Vec<String>>,
}

impl EmailData {
    pub fn new(subject: String) -> Self {
        Self {
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject,
            body_text: None,
            body_html: None,
            in_reply_to: None,
            references: None,
        }
    }

    pub fn add_to(&mut self, email: &str, name: Option<&str>) -> Result<&mut Self> {
        let mailbox = if let Some(n) = name {
            Mailbox::new(Some(n.to_string()), email.parse()?)
        } else {
            Mailbox::new(None, email.parse()?)
        };
        self.to.push(mailbox);
        Ok(self)
    }

    pub fn add_cc(&mut self, email: &str, name: Option<&str>) -> Result<&mut Self> {
        let mailbox = if let Some(n) = name {
            Mailbox::new(Some(n.to_string()), email.parse()?)
        } else {
            Mailbox::new(None, email.parse()?)
        };
        self.cc.push(mailbox);
        Ok(self)
    }

    pub fn add_bcc(&mut self, email: &str, name: Option<&str>) -> Result<&mut Self> {
        let mailbox = if let Some(n) = name {
            Mailbox::new(Some(n.to_string()), email.parse()?)
        } else {
            Mailbox::new(None, email.parse()?)
        };
        self.bcc.push(mailbox);
        Ok(self)
    }

    pub fn with_text_body(mut self, body: String) -> Self {
        self.body_text = Some(body);
        self
    }

    pub fn with_html_body(mut self, body: String) -> Self {
        self.body_html = Some(body);
        self
    }

    pub fn with_in_reply_to(mut self, message_id: String) -> Self {
        self.in_reply_to = Some(message_id);
        self
    }

    pub fn with_references(mut self, references: Vec<String>) -> Self {
        self.references = Some(references);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_data_builder() {
        let mut email = EmailData::new("Test Subject".to_string());
        email.add_to("recipient@example.com", Some("Recipient Name")).unwrap();
        email.add_cc("cc@example.com", None).unwrap();

        let email = email
            .with_text_body("This is the plain text body".to_string())
            .with_html_body("<p>This is the HTML body</p>".to_string());

        assert_eq!(email.to.len(), 1);
        assert_eq!(email.cc.len(), 1);
        assert_eq!(email.subject, "Test Subject");
        assert!(email.body_text.is_some());
        assert!(email.body_html.is_some());
    }
}
