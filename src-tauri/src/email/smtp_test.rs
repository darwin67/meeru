// Simplified SMTP client for testing (no TLS)
use anyhow::{Context, Result};
use lettre::message::{header::ContentType, Mailbox, MultiPart, SinglePart};
use lettre::{Message, SmtpTransport, Transport};

use super::smtp::EmailData;

pub struct SmtpTestClient {
    transport: SmtpTransport,
    from_address: Mailbox,
}

impl SmtpTestClient {
    /// Create a new SMTP client without TLS (for testing)
    pub fn new_plain(host: &str, port: u16, email: &str, from_name: Option<&str>) -> Result<Self> {
        // Create transport without TLS
        let transport = SmtpTransport::builder_dangerous(host)
            .port(port)
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
