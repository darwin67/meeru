//! RFC822 parsing helpers for the generic provider baseline.

use chrono::{DateTime, Utc};
use mail_parser::{MessageParser, MimeHeaders};

use crate::Result;

/// Parsed mailbox address extracted from RFC822 headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEmailAddress {
    /// Normalized email address.
    pub address: String,
    /// Optional display name paired with the address.
    pub name: Option<String>,
}

/// Attachment payload extracted from a parsed RFC822 message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAttachment {
    /// Filename advertised by the attachment metadata, or a fallback name.
    pub filename: String,
    /// Best-effort MIME type assembled from the attachment headers.
    pub mime_type: Option<String>,
    /// Decoded attachment bytes ready for local persistence.
    pub content: Vec<u8>,
}

/// Structured view of the message parts the MVP needs after RFC822 parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMessage {
    /// RFC822 `Message-ID` header when present.
    pub message_id: Option<String>,
    /// Decoded subject line.
    pub subject: Option<String>,
    /// First sender from the `From` header.
    pub from: Option<ParsedEmailAddress>,
    /// Recipient list from the `To` header.
    pub to: Vec<ParsedEmailAddress>,
    /// Parsed message date converted to UTC when possible.
    pub date: Option<DateTime<Utc>>,
    /// Plain-text body chosen by the parser.
    pub text_body: Option<String>,
    /// HTML body chosen by the parser.
    pub html_body: Option<String>,
    /// Non-message attachments collected from the MIME structure.
    pub attachments: Vec<ParsedAttachment>,
}

/// Parse a raw RFC822 message into the normalized fields used by storage and UI layers.
pub fn parse_rfc822_message(raw_message: &[u8]) -> Result<ParsedMessage> {
    let message = MessageParser::default()
        .parse(raw_message)
        .ok_or_else(|| crate::Error::Parse("failed to parse RFC822 message".to_string()))?;

    let text_body = message.body_text(0).map(|body| body.into_owned());
    let html_body = message.body_html(0).map(|body| body.into_owned());
    let attachments = message
        .attachments()
        .filter(|attachment| !attachment.is_message())
        .map(|attachment| ParsedAttachment {
            filename: attachment
                .attachment_name()
                .unwrap_or("attachment.bin")
                .to_string(),
            mime_type: attachment.content_type().map(|content_type| {
                if let Some(subtype) = &content_type.c_subtype {
                    format!("{}/{}", content_type.c_type, subtype)
                } else {
                    content_type.c_type.to_string()
                }
            }),
            content: attachment.contents().to_vec(),
        })
        .collect();

    Ok(ParsedMessage {
        message_id: message.message_id().map(ToOwned::to_owned),
        subject: message.subject().map(ToOwned::to_owned),
        from: message
            .from()
            .and_then(|from| from.first())
            .and_then(address_from_mail),
        to: message
            .to()
            .map(|addresses| addresses.iter().filter_map(address_from_mail).collect())
            .unwrap_or_default(),
        date: message
            .date()
            .and_then(|date| DateTime::parse_from_rfc3339(&date.to_rfc3339()).ok())
            .map(|date| date.with_timezone(&Utc)),
        text_body,
        html_body,
        attachments,
    })
}

fn address_from_mail(address: &mail_parser::Addr<'_>) -> Option<ParsedEmailAddress> {
    Some(ParsedEmailAddress {
        address: address.address.as_ref()?.to_string(),
        name: address.name.as_ref().map(ToString::to_string),
    })
}
