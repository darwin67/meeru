//! RFC822 parsing helpers for the generic provider baseline.

use chrono::{DateTime, Utc};
use mail_parser::{MessageParser, MimeHeaders};

use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEmailAddress {
    pub address: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAttachment {
    pub filename: String,
    pub mime_type: Option<String>,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMessage {
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub from: Option<ParsedEmailAddress>,
    pub to: Vec<ParsedEmailAddress>,
    pub date: Option<DateTime<Utc>>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub attachments: Vec<ParsedAttachment>,
}

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
